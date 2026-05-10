const fs = require('fs');
const https = require('https');

const args = process.argv.slice(2);
const params = {};
args.forEach(arg => {
  const [key, value] = arg.split('=');
  params[key.replace('--', '')] = value;
});

const repo = params.repo || 'cluaiz/cluaiz';
const tag = params.tag;
const templatePath = params.template;
const outputPath = params.output || 'registry-synced.json';
const version = params.version || 'dev-release';
const token = process.env.GITHUB_TOKEN;

if (!tag || !templatePath) {
  console.error("Usage: node verify_manifest.js --tag=driver-v1.0.0 --template=registry.json [--version=v1.0.0]");
  process.exit(1);
}

const getReleaseAssets = () => {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: 'api.github.com',
      path: `/repos/${repo}/releases/tags/${tag}`,
      headers: {
        'User-Agent': 'Cluaiz-CI',
        'Accept': 'application/vnd.github.v3+json'
      }
    };
    if (token) {
      options.headers['Authorization'] = `token ${token}`;
    }

    https.get(options, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        if (res.statusCode === 404) {
          console.warn(`[WARNING] Release tag ${tag} not found. Assuming no assets built yet.`);
          return resolve([]);
        }
        if (res.statusCode !== 200) {
          return reject(new Error(`GitHub API error ${res.statusCode}: ${data}`));
        }
        const release = JSON.parse(data);
        resolve(release.assets.map(a => a.name));
      });
    }).on('error', reject);
  });
};

const processManifest = (manifestStr, assets) => {
  // Global Sovereign Replacements
  const baseReplaced = manifestStr
    .replace(/{VERSION}/g, version)
    .replace(/{DRIVER_TAG}/g, tag)
    .replace(/{KERNEL_TAG}/g, tag)
    .replace(/{ENGINE_TAG}/g, tag)
    .replace(/{CLI_TAG}/g, tag)
    .replace(/{BASE_URL}/g, 'https://github.com/cluaiz/cluaiz/releases/download')
    .replace(/{GEN_TIME}/g, new Date().toISOString());

  const manifest = JSON.parse(baseReplaced);

  const pruneArray = (key) => {
    if (!manifest[key]) return;
    manifest[key] = manifest[key].filter(item => {
      // If flat style (master registry array)
      if (item.download_url) {
        const name = item.download_url.split('/').pop();
        if (assets.includes(name)) {
          return true;
        } else {
          console.warn(`[PRUNED] ❌ Missing asset deleted from JSON: ${name}`);
          return false;
        }
      }

      // If nested artifacts style (inference-drivers)
      if (item.artifacts && Array.isArray(item.artifacts)) {
        item.artifacts = item.artifacts.filter(artifact => {
          if (assets.includes(artifact.name_template)) {
            return true;
          } else {
            console.warn(`[PRUNED] ❌ Missing asset deleted from JSON: ${artifact.name_template}`);
            return false;
          }
        });
        return item.artifacts.length > 0;
      }
      
      return true;
    });
  };

  ['backends', 'kernels', 'drivers', 'engines', 'cli'].forEach(pruneArray);

  return manifest;
};

const main = async () => {
  try {
    console.log(`[VERIFY] Fetching live compiled assets for ${repo}@${tag}...`);
    const assets = await getReleaseAssets();
    console.log(`[VERIFY] Found ${assets.length} live assets on GitHub Release server.`);

    const manifestStr = fs.readFileSync(templatePath, 'utf8');
    const prunedManifest = processManifest(manifestStr, assets);

    fs.writeFileSync(outputPath, JSON.stringify(prunedManifest, null, 2));
    console.log(`[SUCCESS] Wrote 100% Zero-404 verified manifest to ${outputPath}`);
  } catch (err) {
    console.error("[ERROR]", err);
    process.exit(1);
  }
};

main();
