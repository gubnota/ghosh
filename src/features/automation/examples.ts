import type { AutomationDocument } from "../../generated/contracts";

export const examples: { label: string; description: string; document: AutomationDocument }[] = [
  { label: "Inspect Linux node", description: "Read OS, uptime and disk information. The check prints facts and makes no changes.", document: {
    id: "linux-facts", kind: "playbook", source: `schema: 1
name: Inspect Linux node
targets: []
tasks:
  - name: Show system facts
    check:
      command: 'uname -a; uptime; df -h /'
    shell:
      command: 'true'
    timeout_s: 30
` } },
  { label: "Create a temporary directory", description: "A small idempotent example: check returns 10 for drift, apply creates the directory, then check verifies it.", document: {
    id: "temporary-directory", kind: "playbook", source: `schema: 1
name: Create temporary example directory
targets: []
tasks:
  - name: Ensure example directory exists
    check:
      command: 'if [ -d /tmp/ghosh-example ]; then exit 0; else exit 10; fi'
    shell:
      command: 'mkdir -p /tmp/ghosh-example'
    timeout_s: 30
    changed_when: exit_code == 0
` } },
  { label: "Existing Ansible project", description: "Link the existing project. Roles, includes, templates, inventory and variables stay in that folder; no credentials are copied.", document: {
    id: "server-first-run", kind: "ansible", source: `schema: 1
name: server-first-run · SSH setup
project: /Users/vm/Documents/petProjects/server-first-run
playbook: ssh_playbook.yml
inventory: hosts.ini
` } },
  { label: "Shell · system report", description: "A shell script for a local or remote node. Choose targets and execute; results are saved in Runs.", document: {
    id: "system-report", kind: "shell", source: `#!/bin/sh
set -eu
printf 'System report\\n'
uname -a
uptime
df -h /
` } },
  { label: "Python · JSON result", description: "Return structured data for subsequent automation steps.", document: {
    id: "json-result", kind: "python", source: `import json
import platform
import time

print(json.dumps({"host": platform.node(), "system": platform.system(), "time": int(time.time())}))
` } },
  { label: "Deno · process JSON", description: "Optional JavaScript/TypeScript runtime. Deno must already be installed on the selected machine.", document: {
    id: "process-result", kind: "deno", source: `const result = { host: Deno.hostname(), created: new Date().toISOString() };
console.log(JSON.stringify(result, null, 2));
` } },
];
