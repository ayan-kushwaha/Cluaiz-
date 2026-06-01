import os
import re

root = r'C:\Users\Aryan\my\Cluaiz-workspace\cluaiz\test\benchmark\GPU\NVIDIA_GeForce_RTX_3050_A_Laptop_GPU'
files = [os.path.join(dp, f) for dp, dn, filenames in os.walk(root) for f in filenames if f.endswith('.md')]

pattern = re.compile(r'#### 🧠 Thinking Mode: (ON|OFF)\s*- \*\*Speed\*\*: ([\d\.]+) TPS\s*- \*\*TTFT\*\*: ([\d\.]+) s\s*- \*\*Tokens\*\*: (\d+)\s*- \*\*Time\*\*: ([\d\.]+) s')

table = '| Model | Test | Mode | Speed (TPS) | TTFT (s) | Tokens | Time (s) |\n|---|---|---|---|---|---|---|\n'

for f in files:
    content = open(f, encoding='utf-8').read()
    matches = pattern.findall(content)
    model = os.path.basename(os.path.dirname(f))
    test = os.path.basename(f).replace('.md','')
    for m in matches:
        table += f'| {model} | {test} | {m[0]} | {m[1]} | {m[2]} | {m[3]} | {m[4]} |\n'

print(table)
