import type { AutomationDocument } from "../../generated/contracts";

export const examples: { label: string; description: string; document: AutomationDocument }[] = [
  { label: "Simple script with success check", description: "No Ansible required. Check the directory, create it only when missing, then verify success. Run locally or over SSH.", document: {
    id: "checked-script", kind: "playbook", source: `schema: 1
name: Simple script with success check
tasks:
  - name: Create and verify example directory
    check:
      command: test -d /tmp/ghosh-checked-example || exit 10
    shell:
      command: mkdir -p /tmp/ghosh-checked-example
    timeout_s: 30
` } },
  { label: "Inspect Linux node", description: "Read OS, uptime and disk information. The check prints facts and makes no changes.", document: {
    id: "linux-facts", kind: "playbook", source: `- name: Inspect Linux node
  hosts: all
  gather_facts: false
  tasks:
    - name: Show system facts
      ansible.builtin.command: uname -a
      changed_when: false
      check_mode: false
    - name: Read uptime
      ansible.builtin.command: uptime
      changed_when: false
      check_mode: false
` } },
  { label: "Create a temporary directory", description: "Standard Ansible file module: Check predicts changes; Apply creates the directory idempotently.", document: {
    id: "temporary-directory", kind: "playbook", source: `- name: Create temporary example directory
  hosts: all
  gather_facts: false
  tasks:
    - name: Ensure example directory exists
      ansible.builtin.file:
        path: /tmp/ghosh-example
        state: directory
        mode: '0700'
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
  { label: "Python · JSON result", description: "Read GHOSH_INPUT and return a JSON result. Export the saved report from Runs.", document: {
    id: "json-result", kind: "python", source: `import json
import os
import platform
import time

print(json.dumps({"host": platform.node(), "system": platform.system(), "time": int(time.time()), "input": json.loads(os.environ.get("GHOSH_INPUT", "{}"))}))
` } },
  { label: "Deno · process JSON", description: "Optional JavaScript/TypeScript runtime. Deno must already be installed on the selected machine.", document: {
    id: "process-result", kind: "deno", source: `const result = { host: Deno.hostname(), created: new Date().toISOString() };
console.log(JSON.stringify(result, null, 2));
` } },
];
