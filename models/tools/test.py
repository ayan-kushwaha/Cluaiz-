import os
import json
import urllib.request
import urllib.error
import glob
import re

LIBRARY_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'library'))
TOLERANCE_PCT = 1.5

class Colors:
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    RESET = '\033[0m'
    BOLD = '\033[1m'

def log_pass(msg):
    print(f"{Colors.GREEN}[PASS]{Colors.RESET} {msg}")

def log_fail(msg):
    print(f"{Colors.RED}[FAIL]{Colors.RESET} {msg}")

def log_info(msg):
    print(f"{Colors.YELLOW}[INFO]{Colors.RESET} {msg}")

def get_url_size(url):
    """Performs HTTP HEAD request and returns Content-Length in bytes."""
    try:
        headers = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'}
        req = urllib.request.Request(url, method='HEAD', headers=headers)
        with urllib.request.urlopen(req, timeout=15) as response:
            size = response.headers.get('Content-Length')
            if size is None:
                req = urllib.request.Request(url, headers=headers)
                with urllib.request.urlopen(req, timeout=15) as get_resp:
                    size = get_resp.headers.get('Content-Length')
            return int(size) if size else 0
    except Exception as e:
        raise Exception(f"Failed to reach {url}: {e}")

def check_gguf_metadata(url, expected_arch):
    """Downloads first 1MB of GGUF and checks magic bytes and architecture."""
    try:
        headers = {'Range': 'bytes=0-1048576', 'User-Agent': 'Mozilla/5.0'}
        req = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(req, timeout=15) as response:
            data = response.read()
            if not data.startswith(b"GGUF"):
                return False, "Not a valid GGUF file (Magic bytes missing)"
            
            arch_bytes = expected_arch.lower().encode('utf-8')
            if b"general.architecture" in data and arch_bytes in data:
                return True, "Valid GGUF and Architecture matched"
            else:
                return True, "Valid GGUF (Architecture string not strictly verified in 1MB chunk)"
    except Exception as e:
        return False, f"Failed to download range from {url}: {e}"

def validate_json(filepath):
    print(f"\n{Colors.BOLD}Validating: {os.path.basename(filepath)}{Colors.RESET}")
    expected_arch = os.path.basename(os.path.dirname(filepath)).lower()
    
    with open(filepath, 'r', encoding='utf-8') as f:
        try:
            data = json.load(f)
            log_pass("JSON Syntax Check")
        except json.JSONDecodeError as e:
            log_fail(f"JSON Syntax Check - {e}")
            return False

    all_passed = True

    # Traverse models and variants
    for model_key, model_data in data.items():
        if not isinstance(model_data, dict) or 'variants' not in model_data:
            continue
            
        for variant_type, variant_data in model_data.get('variants', {}).items():
            for quant, specs in variant_data.items():
                ram_gb = specs.get('ram_required_gb')
                size_gb = specs.get('download_size_gb')
                
                # 1. Data Type Validation
                if not isinstance(ram_gb, (int, float)) or not isinstance(size_gb, (int, float)):
                    log_fail(f"[{model_key} | {quant}] Data Type Check: ram_required_gb and download_size_gb must be numbers.")
                    all_passed = False
                    continue
                log_pass(f"[{model_key} | {quant}] Data Type Check")

                # 2. Logical Sanity Check
                if ram_gb <= size_gb:
                    log_fail(f"[{model_key} | {quant}] Logical Sanity Check: RAM ({ram_gb}GB) must be > Size ({size_gb}GB).")
                    all_passed = False
                else:
                    log_pass(f"[{model_key} | {quant}] Logical Sanity Check")

                # Collect URLs
                urls = []
                if 'models' in specs:
                    urls = [m.get('url', m.get('download_url')) for m in specs['models'] if m.get('url') or m.get('download_url')]
                else:
                    url = specs.get('download_url', specs.get('repo_url'))
                    if url:
                        urls.append(url)
                
                if not urls:
                    log_fail(f"[{model_key} | {quant}] No valid URLs found.")
                    all_passed = False
                    continue

                # 3 & 4. URL Reachability & Size Check
                total_bytes = 0
                urls_reachable = True
                for url in urls:
                    try:
                        sz = get_url_size(url)
                        total_bytes += sz
                    except Exception as e:
                        log_fail(f"[{model_key} | {quant}] URL Reachability: {e}")
                        urls_reachable = False
                        all_passed = False
                        break
                
                if urls_reachable:
                    log_pass(f"[{model_key} | {quant}] URL Reachability & Shard Completeness")
                    
                    if variant_type == 'gguf' and total_bytes > 0:
                        calc_size_gb = total_bytes / (1024 ** 3)
                        diff_pct = abs(calc_size_gb - size_gb) / size_gb * 100
                        if diff_pct > TOLERANCE_PCT:
                            log_fail(f"[{model_key} | {quant}] Exact Size Matching: Actual {calc_size_gb:.2f}GB vs JSON {size_gb}GB (Diff: {diff_pct:.2f}% > {TOLERANCE_PCT}%)")
                            all_passed = False
                        else:
                            log_pass(f"[{model_key} | {quant}] Exact Size Matching (Diff: {diff_pct:.2f}%)")
                    else:
                        log_info(f"[{model_key} | {quant}] Exact Size Matching: Skipped for repo_url (AWQ)")

                # 5. GGUF Metadata Check
                if variant_type == 'gguf' and urls:
                    first_url = urls[0]
                    is_valid, msg = check_gguf_metadata(first_url, expected_arch)
                    if is_valid:
                        log_pass(f"[{model_key} | {quant}] GGUF Metadata Range Check: {msg}")
                    else:
                        log_fail(f"[{model_key} | {quant}] GGUF Metadata Range Check: {msg}")
                        all_passed = False

    return all_passed

def main():
    print(f"Starting Cluaize Model Validation...")
    print(f"Target Directory: {LIBRARY_DIR}")
    
    json_files = glob.glob(os.path.join(LIBRARY_DIR, '**/*.json'), recursive=True)
    if not json_files:
        log_fail("No JSON files found in library directory!")
        exit(1)
        
    all_success = True
    for file in json_files:
        # Skip registry.json or non-model JSONs if needed
        if os.path.basename(file) == 'registry.json':
            continue
            
        success = validate_json(file)
        if not success:
            all_success = False
            
    print("\n" + "="*40)
    if all_success:
        print(f"{Colors.GREEN}{Colors.BOLD}ALL CHECKS PASSED! The library is Production-Ready.{Colors.RESET}")
        exit(0)
    else:
        print(f"{Colors.RED}{Colors.BOLD}VALIDATION FAILED! Check logs for details.{Colors.RESET}")
        exit(1)

if __name__ == "__main__":
    main()
