import { test, expect, type Page, type Locator } from "@playwright/test";

// Browser-only deterministic IPC fixture. This does not exercise native services.
async function fixture(page: Page, trusted = true, unknown = false, credential = false, schema = 1) {
  await page.route("**/", async route => {
    const response = await route.fetch();
    const body = (await response.text()).replace(/<script type="module" src="\/src\/main\.tsx(?:\?[^"]*)?"><\/script>/, `<script type="module">
      import { mockIPC, mockWindows } from '/node_modules/@tauri-apps/api/mocks.js';
      window.isTauri = true; mockWindows('main');
      window.fixtureCalls = [];
      window.fixtureTerminalInput = [];
      window.fixtureTerminalAck = [];
      window.fixtureTerminalCount = 0;
      window.fixtureAutomations = [];
      window.fixtureRuns = [];
      window.fixtureKeys = {revision:'keys-1',default_key:null,keys:[]};
      window.fixtureWorkspace = { id: 'fixture', name: 'Test fleet', path: '/tmp/Test.ghosh', revision: 'r1', schema: ${schema}, read_only: false, trusted: ${trusted}, external_changed: false, notices: [], nodes: [
        {id:'edge1',host:'192.0.2.1',port:22,user:'admin',labels:['web']}, {id:'db1',host:'192.0.2.2',port:22,user:'deploy',labels:['database']}
      ]};
      window.fixtureSavedWorkspace = structuredClone(window.fixtureWorkspace);
      window.fixtureRecent = [{name:'Test fleet',path:'/tmp/Test.ghosh'},{name:'Other fleet',path:'/tmp/Other.ghosh'}];
      window.fixtureOperations = { jobs: { sequence: 1, jobs: [{id:'job-1',kind:'workspace_open',state:'succeeded',cancellable:false,started_at:Date.now(),finished_at:Date.now(),error:null,stages:['started','completed']}] }, approvals: { sequence: 1, requests: [], audit: [] } };
      mockIPC(async (cmd, args) => {
        window.fixtureCalls.push(cmd);
        if (cmd === 'playbook_export') { window.fixtureExport = structuredClone(args.document); return true; }
        if (cmd === 'playbook_import') return structuredClone(window.fixtureImport || null);
        if (cmd === 'notebook_parse') return JSON.parse(args.source);
        if (cmd === 'system_key_candidates') return ['System Ed25519','OrbStack Linux'].map((label,i)=>({request:{label,path:i?'/fixture/.orbstack/ssh/id_ed25519':'/fixture/.ssh/id_ed25519',storage:'reference'},digest:'review-'+i,public:{algorithm:'ED25519',fingerprint:'SHA256:system-'+i},encrypted:false}));
        if (cmd === 'runs_cleanup') {const before=window.fixtureRuns.length;window.fixtureRuns=window.fixtureRuns.filter(run=>run.state==='running'||(args.request.id&&run.id!==args.request.id));return before-window.fixtureRuns.length;}
        if (cmd === 'runs_list') return structuredClone(window.fixtureRuns);
        if (cmd === 'run_start') {const request=args.request;const report={id:'run-fixture',job_id:'execution-job',workspace_id:'fixture',revision:request.automation.context.revision,automation_id:request.automation.id,kind:request.automation.kind,mode:request.mode,targets:request.targets.map(target=>target.node_id),state:'running',started_at:Date.now(),finished_at:null,results:[],message:'Targets execute sequentially.'};window.fixtureRunRequest=structuredClone(request);window.fixtureRuns=[report];window.fixtureOperations.jobs.jobs.push({id:'execution-job',kind:'automation_run',state:'running',cancellable:true,started_at:Date.now(),finished_at:null,error:null,stages:['started']});return structuredClone(report);}
        if (cmd === 'run_stop') {window.fixtureRuns[0].state='outcome_unknown';window.fixtureRuns[0].message='Inspect the node before retrying.';window.fixtureRuns[0].results=[{node_id:'edge1',name:'Ensure',phase:'apply',state:'outcome_unknown',output:'very-long-output-'.repeat(120),exit_code:null,truncated:false}];window.fixtureOperations.jobs.jobs.find(job=>job.id==='execution-job').state='cancelled';return null;}
        if (cmd === 'node_connection_preview') {if(window.fixtureConnectionError && args.request.node_id===window.fixtureConnectionError.node) throw window.fixtureConnectionError.error;return {ticket:'review-fixture',node_id:args.request.node_id,host:'192.0.2.1',port:22,user:'admin',identity:{source:'node',key_id:'fixture-key',public:{algorithm:'ED25519',fingerprint:'SHA256:identity-fixture'},status:'available',registry_revision:'keys-1'},host_key:{algorithm:'ED25519',fingerprint:'SHA256:host-fixture',trust:${unknown}?'unknown':'known'},credential_prompt:${credential}?'Key passphrase':null};}
        if (cmd === 'node_connection_test') {window.fixtureAuthentication = args.request;return {node_id:'edge1',host_key:{algorithm:'ED25519',fingerprint:'SHA256:host-fixture',trust:'known'},elapsed_ms:42};}
        if (cmd === 'terminal_open_node') {window.fixtureAuthentication = args.request.connection;return {id:'remote-fixture-'+(++window.fixtureTerminalCount),target:'edge1 · admin@192.0.2.1:22',shell:'SSH login shell',cwd:'~'};}
        if (cmd === 'terminal_open_local') return {id:'pty-fixture-'+(++window.fixtureTerminalCount),target:'Local machine',shell:'/bin/zsh',cwd:'/fixture/home'};
        if (cmd === 'terminal_poll') {window.fixtureTerminalAck.push(args.ack); return args.ack === 0 ? {sequence:1,bytes:Array.from(new TextEncoder().encode('REAL_PTY_FIXTURE 世界\\r\\n')),exited:false,exit_code:null,failed:false} : {sequence:1,bytes:[],exited:false,exit_code:null,failed:false};}
        if (cmd === 'terminal_input') {window.fixtureTerminalInput.push(...args.bytes);window.fixtureLastInputSession=args.id;return null;}
        if (cmd === 'monitor_open') {window.fixtureMonitorRequest=args.request;return {id:'monitor-fixture',target:args.request.connection?'edge1':'Local machine'};}
        if (cmd === 'monitor_poll') {window.fixtureMonitorVisible=args.visible;return {output:window.fixtureMonitorOutput||'',running:!!window.fixtureMonitorRunning,exit_code:null,truncated:false,metrics:{cpu_percent:12.5,memory_percent:40,disk_percent:23}};}
        if (cmd === 'monitor_start') {window.fixtureMonitorCommand=args.command;window.fixtureMonitorRunning=true;window.fixtureMonitorOutput='monitor live fixture';return null;}
        if (cmd === 'monitor_close') {window.fixtureMonitorRunning=false;return null;}
        if (cmd === 'terminal_resize' || cmd === 'terminal_close') return null;
        if (cmd === 'workspace_key') return window.fixtureWorkspace.default_key || null;
        if (cmd === 'workspace_key_preview') return {digest:'migration-review',before_schema:window.fixtureWorkspace.schema,before_default:null,after_default:args.request.default_key,backup:'.ghosh-manifest-r1.backup',legacy_nodes:['old-key-node']};
        if (cmd === 'workspace_key_commit') {window.fixtureWorkspace.schema=2;window.fixtureWorkspace.default_key=args.request.request.default_key;window.fixtureWorkspace.revision+='-identity';return structuredClone(window.fixtureWorkspace);}
        if (cmd === 'node_identity') {const id=args.request.node.auth.key_id || window.fixtureWorkspace.default_key || window.fixtureKeys.default_key;const key=window.fixtureKeys.keys.find(key=>key.id===id);return {source:args.request.node.auth.key_id?'node':window.fixtureWorkspace.default_key?'workspace':'global',key_id:id,public:key?.public||null,status:key?.status||'missing',registry_revision:window.fixtureKeys.revision};}
        if (cmd === 'keys_generate') {window.fixtureKeys.keys.push({id:'generated-fixture',label:args.request.label,path:'/fixture/managed/identity',storage:'managed',public:{algorithm:'ED25519',fingerprint:'SHA256:generated-fixture'},encrypted:false,status:'available'});window.fixtureKeys.revision+='-generate';return structuredClone(window.fixtureKeys);}
        if (cmd === 'keys_relink') {const key=window.fixtureKeys.keys.find(key=>key.id===args.request.id);key.path=args.request.preview.request.path;window.fixtureKeys.revision+='-relink';return structuredClone(window.fixtureKeys);}
        if (cmd === 'keys_list') return structuredClone(window.fixtureKeys);
        if (cmd === 'keys_preview') return {request:args.request,digest:'fixture-key-review',public:{algorithm:'ED25519',fingerprint:'SHA256:public-fixture'},encrypted:true};
        if (cmd === 'keys_commit') {
          if (args.request.revision !== window.fixtureKeys.revision) throw {code:'conflict',message:'The key library changed. Refresh keys and review again.'};
          const review = args.request.preview;
          window.fixtureKeys.keys.push({id:window.fixtureKeys.keys.length?'key-fixture-'+window.fixtureKeys.keys.length:'key-fixture',label:review.request.label,path:review.request.path,storage:review.request.storage,public:review.public,encrypted:review.encrypted,status:'available'});
          window.fixtureKeys.revision += '-save'; return structuredClone(window.fixtureKeys);
        }
        if (cmd === 'keys_default') { window.fixtureKeys.default_key = args.request.id; window.fixtureKeys.revision += '-default'; return structuredClone(window.fixtureKeys); }
        if (cmd === 'keys_forget') { window.fixtureKeys.keys = []; window.fixtureKeys.default_key = null; window.fixtureKeys.revision += '-remove'; return structuredClone(window.fixtureKeys); }
        if (cmd === 'operations_snapshot') return structuredClone(window.fixtureOperations);
        if (cmd === 'approvals_decide') {
          const request = window.fixtureOperations.approvals.requests.find(request => request.id === args.request.id);
          if (request.state !== 'pending' && args.request.decision !== 'revoke') throw {code:'denied',message:'This request can no longer be approved.'};
          request.state = ({allow_once:'allowed_once',allow_session:'allowed_session',deny:'denied',revoke:'invalidated'})[args.request.decision];
          window.fixtureOperations.approvals.sequence++;
          return null;
        }
        if (cmd === 'automations_list') return structuredClone(window.fixtureAutomations.map(document => ({id:document.id,kind:document.kind,validation:{valid:true,name:document.id,task_count:1,issues:[]}})));
        if (cmd === 'automation_lint') return {engine:'Fixture syntax diagnostics',diagnostics:args.document.source.includes('if then')?[{line:1,column:1,message:'Expected condition'}]:[]};
        if (cmd === 'automation_read') return structuredClone(window.fixtureAutomations.find(document => document.id === args.request.id && document.kind === args.request.kind));
        if (cmd === 'automation_preview') {const valid = args.request.remove || (args.request.document.source.includes('schema: 1') || args.request.document.source.includes('hosts: all')) || ['shell','python','deno','notebook'].includes(args.request.document.kind);return {digest:'automation-fixture',validation:{valid,name:args.request.document.id,task_count:1,issues:valid?[]:['Invalid native YAML.']}};}
        if (cmd === 'automation_commit') {const mutation=args.request.mutation;window.fixtureAutomations=window.fixtureAutomations.filter(document => document.id !== mutation.document.id || document.kind !== mutation.document.kind);if(!mutation.remove)window.fixtureAutomations.push(structuredClone(mutation.document));window.fixtureWorkspace.revision+='-automation';return structuredClone(window.fixtureWorkspace);}
        if (cmd === 'nodes_list') return window.fixtureWorkspace.nodes.map(node => ({...node,schema:node.schema||1,auth:node.auth||{method:'agent',key_ref:null,key_id:null,password_ref:null,agent_socket:null},tunnel:node.tunnel||'direct',jump:node.jump||null}));
        if (cmd === 'nodes_preview') {
          const change = args.request.change;
          const before = window.fixtureWorkspace.nodes.find(node => node.id === (change.node?.id || change.id)) || null;
          return { digest: 'fixture-preview', before, after: change.kind === 'remove' ? null : change.node };
        }
        if (cmd === 'nodes_commit') {
          const change = args.request.mutation.change;
          window.fixtureWorkspace.nodes = window.fixtureWorkspace.nodes.filter(node => node.id !== (change.node?.id || change.id));
          if (change.kind !== 'remove') window.fixtureWorkspace.nodes.push(change.node);
          window.fixtureWorkspace.revision += '-nodes'; window.fixtureWorkspace.trusted = true;
          return structuredClone(window.fixtureWorkspace);
        }
        if (cmd === 'workspace_startup') return structuredClone(window.fixtureWorkspace);
        if (cmd === 'workspace_close') {window.fixtureWorkspace = null; return null;}
        if (cmd === 'workspace_forget_recent') {window.fixtureRecent = args.path ? window.fixtureRecent.filter(entry => entry.path !== args.path) : []; return structuredClone(window.fixtureRecent);}
        if (cmd === 'workspace_open') {window.fixtureWorkspace = structuredClone(window.fixtureSavedWorkspace); return structuredClone(window.fixtureWorkspace);}
        if (cmd === 'workspace_snapshot') return structuredClone(window.fixtureWorkspace);
        if (cmd === 'workspace_recent') return structuredClone(window.fixtureRecent);
        if (cmd === 'workspace_pending') return null;
        if (cmd === 'workspace_rename') { window.fixtureWorkspace.name = args.request.name; window.fixtureWorkspace.revision = 'r2'; return structuredClone(window.fixtureWorkspace); }
        if (cmd === 'workspace_reload') { window.fixtureWorkspace.external_changed = false; return structuredClone(window.fixtureWorkspace); }
        if (cmd === 'workspace_trust') { window.fixtureWorkspace.trusted = args.request.trusted; return structuredClone(window.fixtureWorkspace); }
        if (cmd === 'plugin:window|close') return null;
        throw new Error('Unimplemented fixture command: ' + cmd);
      }, { shouldMockEvents: true });
      await import('/src/main.tsx');
    </script>`);
    await route.fulfill({ response, body });
  });
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Test fleet", exact: true })).toBeVisible();
}
async function command(page: Page, text: string) {
  await page.keyboard.press("Meta+k");
  await page.getByRole("combobox", { name: "Search commands" }).fill(text);
  await page.keyboard.press("Enter");
}

async function choose(control: Locator, label: string) {
  await control.click();
  await control.page().getByRole("option", { name: label, exact: true }).click();
}

test("commands trap focus, restore invoker and refuse unavailable/unknown actions", async ({ page }) => {
  await fixture(page);
  const input = page.getByLabel("Workspace name", { exact: true });
  await input.focus();
  await page.keyboard.press("Meta+k");
  const popup = page.getByRole("dialog", { name: "Command popup" });
  await expect(page.getByRole("combobox")).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  await expect(page.getByRole("combobox")).toBeFocused();
  await page.getByRole("combobox").fill("certificate renew");
  await page.keyboard.press("Enter");
  await expect(popup).toBeVisible();
  await expect(popup.getByRole("status")).toContainText("not available");
  await page.getByRole("combobox").fill("run apply nonexistent --target unknown");
  await expect(popup).toContainText("No matching command");
  await page.keyboard.press("Enter");
  await expect(popup).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(input).toBeFocused();
  await command(page, "workspace create");
  await expect(page.getByRole("dialog").getByLabel("Workspace name", { exact: true })).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(input).toBeFocused();
  await page.keyboard.press("Meta+o");
  await expect(page.getByRole("dialog").getByLabel("Workspace location", { exact: true })).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(input).toBeFocused();
  await command(page, "keyboard");
  await expect(page.getByRole("dialog", { name: "Keyboard shortcuts" })).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(input).toBeFocused();
});

test("keyboard tab navigation preserves draft and Ctrl+Tab; dirty close needs confirmation", async ({ page }) => {
  await fixture(page);
  await page.getByLabel("Workspace name", { exact: true }).fill("Unsaved fleet");
  await command(page, "settings");
  await expect(page.getByRole("tab", { name: "Settings", exact: true })).toHaveAttribute("aria-selected", "true");
  await page.keyboard.press("Control+Shift+Tab");
  await expect(page.getByLabel("Workspace name", { exact: true })).toHaveValue("Unsaved fleet");
  await page.keyboard.press("Meta+w");
  await expect(page.getByRole("alertdialog")).toBeVisible();
  await expect(page.getByRole("button", { name: "Cancel", exact: true })).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(page.getByLabel("Workspace name", { exact: true })).toHaveValue("Unsaved fleet");
  await page.keyboard.press("Meta+s");
  await expect(page.getByRole("heading", { name: "Unsaved fleet", exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Save changes" })).toBeDisabled();
  await page.keyboard.press("Control+Tab");
  await page.keyboard.press("Meta+w");
  await expect(page.getByRole("tab", { name: "Overview", exact: true })).toBeFocused();
});

test("external-change event retains draft and blocks shortcut save/trust", async ({ page }) => {
  await fixture(page);
  await page.getByLabel("Workspace name", { exact: true }).fill("Keep this draft");
  await page.evaluate(async () => {
    // @ts-expect-error test fixture only
    window.fixtureWorkspace.external_changed = true;
    const { emit } = await import("/node_modules/@tauri-apps/api/event.js");
    await emit("workspace-files-changed", { workspace_id: "fixture", revision: "external" });
  });
  await expect(page.getByText("This workspace changed outside Ghosh.")).toBeVisible();
  await page.keyboard.press("Meta+s");
  await expect(page.getByLabel("Workspace name", { exact: true })).toHaveValue("Keep this draft");
  await expect(page.getByRole("button", { name: "Save changes" })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Pause operations", exact: true })).toBeDisabled();
  expect(await page.evaluate(() => (window as unknown as { fixtureCalls: string[] }).fixtureCalls.filter(x => x === "workspace_rename"))).toEqual([]);
  await page.getByRole("button", { name: "Reload workspace", exact: true }).click();
  await page.keyboard.press("Escape");
  await expect(page.getByLabel("Workspace name", { exact: true })).toHaveValue("Keep this draft");
});

test("surface search, checkbox selection and persisted appearance", async ({ page }) => {
  await fixture(page);
  await page.keyboard.press("Meta+Shift+e");
  await expect(page.getByRole("button", { name: "Overview", exact: true })).toBeFocused();
  await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  await page.keyboard.press("Meta+f");
  await expect(page.getByRole("textbox", { name: "Filter nodes" })).toBeFocused();
  await page.keyboard.type("web");
  await expect(page.getByRole("checkbox", { name: "Select edge1", exact: true })).toBeVisible();
  await expect(page.getByRole("checkbox", { name: "Select db1", exact: true })).toHaveCount(0);
  await page.getByRole("checkbox", { name: "Select all visible nodes", exact: true }).focus(); await page.keyboard.press("Space");
  await expect(page.getByRole("checkbox", { name: "Select edge1", exact: true })).toBeChecked();
  await command(page, "settings");
  await choose(page.getByLabel("Font size", { exact: true }), "16px");
  await choose(page.getByLabel("Interface zoom", { exact: true }), "200%");
  await expect(page.locator("html")).toHaveCSS("font-size", "32px");
  await expect(page.locator("html")).toHaveAttribute("data-layout-tiny", "");
  // Zoom must reflow columns, not merely enlarge text beyond the viewport.
  const fits = async () => page.evaluate(() => {
    const panel = document.querySelector(".settings-layout .panel")!.getBoundingClientRect();
    const nav = document.querySelector(".settings-layout nav")!.getBoundingClientRect();
    return panel.x >= 0 && panel.right <= innerWidth && panel.y >= nav.bottom;
  });
  expect(await fits()).toBe(true);
  await page.setViewportSize({width: 820, height: 600});
  expect(await fits()).toBe(true);
  await page.getByRole("button", {name: "Reset appearance", exact:true}).focus();
  await page.keyboard.press("Enter");
  await expect(page.locator("html")).toHaveCSS("font-size", "14px");
  await choose(page.getByLabel("Font size", {exact:true}), "16px");
  await choose(page.getByLabel("Interface zoom", {exact:true}), "200%");
  await page.reload();
  await expect(page.locator("html")).toHaveCSS("font-size", "32px");
});


test("native jobs snapshot and approval review are keyboard accessible; stale snapshots cannot revive grants", async ({ page }) => {
  await fixture(page);
  await command(page, "jobs");
  await expect(page.getByRole("heading", { name: "Background jobs", exact: true })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Open workspace", exact: true })).toBeVisible();
  await expect(page.getByText("succeeded", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Cancel job", exact: true })).toHaveCount(0);
  await page.keyboard.press("Meta+f");
  await expect(page.getByLabel("Filter jobs")).toBeFocused();
  await page.evaluate(async () => {
    // @ts-expect-error deterministic browser fixture only
    window.fixtureOperations.approvals = { sequence: 2, requests: [{id:'approval-1',binding:{context:{workspace_id:'test-workspace',revision:'1'.repeat(64),session_id:'test-session'},requester:'mcp',tool:'node.run',source_digest:'2'.repeat(64),targets:['edge1'],route_digest:'3'.repeat(64),identity_digest:'4'.repeat(64)},state:'pending',expires_at:Date.now()+300000}], audit:[] };
    const { emit } = await import("/node_modules/@tauri-apps/api/event.js"); await emit("operations-changed");
  });
  await command(page, "approvals");
  const trigger = page.getByRole("button", { name: "Review node.run" });
  await trigger.focus(); await page.keyboard.press("Enter");
  const modal = page.getByRole("alertdialog", { name: "node.run" });
  await expect(modal.getByRole("button", { name: "Cancel", exact: true })).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  await expect(modal.getByRole("button", { name: "Allow for session", exact: true })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(modal).toContainText("allowed session");
  await expect(modal.getByRole("button", { name: "Allow once", exact: true })).toHaveCount(0);
  await modal.getByRole("button", { name: "Revoke grant", exact: true }).focus(); await page.keyboard.press("Enter");
  await expect(modal).toContainText("invalidated");
  await expect(modal.getByRole("button", { name: "Revoke grant", exact: true })).toHaveCount(0);
  await page.evaluate(async () => {
    // @ts-expect-error deterministic browser fixture only
    window.fixtureOperations.approvals.sequence = 1;
    // @ts-expect-error deterministic browser fixture only
    window.fixtureOperations.approvals.requests[0].state = 'pending';
    const { emit } = await import("/node_modules/@tauri-apps/api/event.js"); await emit("operations-changed");
  });
  await page.getByRole("alertdialog").getByRole("button", {name:"Cancel",exact:true}).focus();
  await page.keyboard.press("Escape");
  await expect(trigger).toBeFocused();
  await expect(page.getByText("invalidated", {exact:true})).toBeVisible();
  await page.reload();
  await expect(page.getByRole("heading", {name:"Test fleet",exact:true})).toBeVisible();
  await command(page,"approvals");
  await expect(page.getByRole("heading", {name:"No approval requests"})).toBeVisible();
});

test("node IDs validate before review and preserve uppercase names and jump references", async ({ page }) => {
  await fixture(page);
  await command(page, "node add");
  const modal = page.getByRole("dialog", { name: "Add node", exact: true });
  const id = modal.getByLabel("Node ID", { exact: true });
  await modal.getByLabel("Host", { exact: true }).fill("192.0.2.30");
  await modal.getByLabel("SSH user", { exact: true }).fill("ops");
  for (const invalid of ["_GPU", "-GPU", "GPU name", "../GPU", "GПU"]) {
    await id.fill(invalid);
    await modal.getByRole("button", { name: "Review change" }).click();
    await expect(modal).toBeVisible();
    expect(await id.evaluate((input: HTMLInputElement) => input.validity.patternMismatch)).toBe(true);
  }
  expect(await page.evaluate(() => (window as unknown as {fixtureCalls:string[]}).fixtureCalls.includes("nodes_preview"))).toBe(false);
  await id.fill("A".repeat(64));
  expect(await id.evaluate((input: HTMLInputElement) => input.checkValidity())).toBe(true);
  await id.fill("RTX3070");
  await modal.getByRole("button", { name: "Review change" }).click();
  const review = page.getByRole("alertdialog", { name: "Review node change" });
  await expect(review).toContainText("Save RTX3070 in this workspace.");
  await review.getByRole("button", { name: "Save node", exact: true }).click();
  await expect(page.getByRole("button", { name: "Edit RTX3070", exact: true })).toBeVisible();
  await command(page, "node add");
  await id.fill("Worker_1-GPU");
  await modal.getByLabel("Host", { exact: true }).fill("192.0.2.31");
  await modal.getByLabel("SSH user", { exact: true }).fill("ops");
  await modal.getByLabel("Jump node ID (optional)").fill("RTX3070");
  await modal.getByRole("button", { name: "Review change" }).click();
  await expect(review.getByRole("row", { name: "Jump node — RTX3070" })).toBeVisible();
  await review.getByRole("button", { name: "Save node", exact: true }).click();
  await page.getByRole("button", { name: "Edit Worker_1-GPU", exact: true }).click();
  const edit = page.getByRole("dialog", { name: "Edit node", exact: true });
  await expect(edit.getByLabel("Node ID", { exact: true })).toHaveValue("Worker_1-GPU");
  await expect(edit.getByLabel("Jump node ID (optional)")).toHaveValue("RTX3070");
});

test("node drafts review exact changes, preserve workspace draft, and block stale commits", async ({ page }) => {
  await fixture(page);
  await page.getByLabel("Workspace name", { exact: true }).fill("Keep overview draft");
  await command(page, "node add");
  const modal = page.getByRole("dialog", { name: "Add node", exact: true });
  await expect(modal.getByLabel("Node ID", { exact: true })).toBeFocused();
  await page.keyboard.type("worker1");
  await page.keyboard.press("Tab"); await page.keyboard.type("192.0.2.10");
  await page.keyboard.press("Tab"); await page.keyboard.type("ops");
  await page.keyboard.press("Meta+s");
  const review = page.getByRole("alertdialog", { name: "Review node change" });
  await expect(review).toContainText("192.0.2.10:22");
  await expect(review.getByRole("button", {name:"Back to draft"})).toBeFocused();
  expect(await page.evaluate(() => (window as unknown as {fixtureCalls:string[]}).fixtureCalls.filter(value => value === "nodes_commit").length)).toBe(0);
  await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  await expect(page.getByRole("button", { name: "Edit worker1", exact: true })).toBeVisible();
  await page.keyboard.press("Control+Shift+Tab");
  await expect(page.getByLabel("Workspace name", {exact:true})).toHaveValue("Keep overview draft");
  await command(page, "node edit");
  const edit = page.getByRole("dialog", {name:"Edit node",exact:true});
  await choose(edit.getByLabel("Node to edit"), "worker1");
  await edit.getByLabel("Host", {exact:true}).fill("192.0.2.11");
  await page.keyboard.press("Escape");
  const discard = page.getByRole("alertdialog", {name:"Discard this node draft?"});
  await expect(discard.getByRole("button", {name:"Keep editing"})).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(edit.getByLabel("Host", {exact:true})).toHaveValue("192.0.2.11");
  await edit.getByRole("button", {name:"Review change"}).focus(); await page.keyboard.press("Enter");
  await page.evaluate(async () => {
    // @ts-expect-error deterministic browser IPC fixture only
    window.fixtureWorkspace.external_changed = true;
    const { emit } = await import("/node_modules/@tauri-apps/api/event.js");
    await emit("workspace-files-changed", {workspace_id:"fixture",revision:"changed"});
  });
  await expect(review.getByRole("button", {name:"Save node",exact:true})).toBeDisabled();
  await expect(review.getByRole("alert")).toContainText("workspace changed");
  await page.keyboard.press("Escape"); await page.keyboard.press("Escape");
  await page.getByRole("button", {name:"Discard draft",exact:true}).focus(); await page.keyboard.press("Enter");
  await command(page,"node add");
  await expect(page.getByRole("dialog", {name:"Command popup"})).toContainText("Reload this workspace");
});

test("file-key flow reviews fingerprints before registration and confirms global changes by keyboard", async ({ page }) => {
  await fixture(page);
  await command(page, "key system");
  const draft = page.getByRole("dialog", {name:"Use system SSH key",exact:true});
  await expect(draft.getByLabel("Key label", {exact:true})).toBeFocused();
  await page.keyboard.type("Operations");
  await page.keyboard.press("Tab"); await page.keyboard.type("/Users/fixture/.ssh/operations");
  await page.keyboard.press("Meta+s");
  const review = page.getByRole("alertdialog", {name:"Review SSH identity",exact:true});
  await expect(review).toContainText("SHA256:public-fixture");
  await expect(review).toContainText("Keep existing file in place");
  await expect(review.getByRole("button", {name:"Back to draft"})).toBeFocused();
  expect(await page.evaluate(() => (window as unknown as {fixtureCalls:string[]}).fixtureCalls.includes("keys_commit"))).toBe(false);
  await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  const makeDefault = page.getByRole("button", {name:"Make default",exact:true});
  await expect(makeDefault).toBeVisible();
  await makeDefault.focus(); await page.keyboard.press("Enter");
  const confirm = page.getByRole("alertdialog", {name:"Change the global default?",exact:true});
  await expect(confirm.getByRole("button", {name:"Cancel",exact:true})).toBeFocused();
  await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  await expect(page.getByText("global default", {exact:true})).toBeVisible();
  await page.getByRole("button", {name:"Remove reference Operations",exact:true}).focus(); await page.keyboard.press("Enter");
  const remove = page.getByRole("alertdialog", {name:"Remove this key reference?",exact:true});
  await expect(remove).toContainText("key file is retained");
  await page.keyboard.press("Escape");
  await expect(page.getByRole("button", {name:"Remove reference Operations",exact:true})).toBeFocused();
  await page.keyboard.press("Enter"); await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  await expect(page.getByRole("heading", {name:"No SSH keys registered",exact:true})).toBeVisible();
  await page.keyboard.press("Meta+n");
  await expect(page.getByRole("dialog", {name:"Import custom SSH key",exact:true}).getByLabel("Key label")).toBeFocused();
  await page.keyboard.type("Stale"); await page.keyboard.press("Tab"); await page.keyboard.type("/Users/fixture/.ssh/stale"); await page.keyboard.press("Meta+s");
  await page.evaluate(() => { (window as unknown as {fixtureKeys:{revision:string}}).fixtureKeys.revision = "external-key-edit"; });
  await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  await expect(review.getByRole("alert")).toContainText("library changed");
  await expect(page.getByRole("heading", {name:"No SSH keys registered",exact:true})).toHaveCount(1);
});

test("key generation and workspace migration require explicit keyboard review", async ({ page }) => {
  await fixture(page);
  await command(page, "key generate");
  await expect(page.getByRole("dialog", {name:"Generate SSH key",exact:true}).getByLabel("Key label")).toBeFocused();
  await page.keyboard.type("Fresh identity"); await page.keyboard.press("Meta+s");
  const generation = page.getByRole("alertdialog", {name:"Generate this SSH identity?",exact:true});
  await expect(generation).toContainText("without a passphrase");
  expect(await page.evaluate(() => (window as unknown as {fixtureCalls:string[]}).fixtureCalls.includes("keys_generate"))).toBe(false);
  await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  await expect(page.getByRole("heading", {name:"Fresh identity",exact:true})).toBeVisible();
  await page.getByRole("button", {name:"Workspace identity…",exact:true}).focus(); await page.keyboard.press("Enter");
  await expect(page.getByLabel("Workspace default key", {exact:true})).toBeFocused();
  await choose(page.getByLabel("Workspace default key", {exact:true}), "Fresh identity · available"); await page.keyboard.press("Meta+s");
  const migration = page.getByRole("alertdialog", {name:"Review workspace identity",exact:true});
  await expect(migration).toContainText("SHA256:generated-fixture");
  await expect(migration).toContainText("old-key-node");
  await expect(migration).toContainText(".ghosh-manifest-r1.backup");
  expect(await page.evaluate(() => (window as unknown as {fixtureCalls:string[]}).fixtureCalls.includes("workspace_key_commit"))).toBe(false);
  await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  await expect(migration).toHaveCount(0);
  expect(await page.evaluate(() => (window as unknown as {fixtureWorkspace:{schema:number;default_key:string}}).fixtureWorkspace.schema)).toBe(2);
  await command(page, "node add");
  await expect(page.getByLabel("Authentication", {exact:true})).toHaveText("System keys (default)");
  await choose(page.getByLabel("Authentication", {exact:true}), "File key");
  await expect(page.getByLabel("SSH identity", {exact:true})).toHaveText("Inherit workspace / global default");
  await expect(page.getByText(/Workspace: SHA256:generated-fixture/)).toBeVisible();
  await page.getByLabel("Node ID", {exact:true}).fill("inherited");
  await page.getByLabel("Host", {exact:true}).fill("192.0.2.30");
  await page.getByLabel("SSH user", {exact:true}).fill("ops");
  await page.keyboard.press("Meta+s");
  const nodeReview = page.getByRole("alertdialog", {name:"Review node change"});
  await expect(nodeReview).toContainText("Identity: workspace · SHA256:generated-fixture");
  await nodeReview.getByRole("button", {name:"Save node",exact:true}).click();
  const saved = await page.evaluate(() => (window as any).fixtureWorkspace.nodes.find((node: any) => node.id === "inherited"));
  expect(saved.schema).toBe(2);
  expect(saved.auth.method).toBe("key");
  expect(saved.auth.key_id).toBeNull();
});

test("relink reviews the existing fingerprint and returns focus without creating a new identity", async ({ page }) => {
  await fixture(page);
  await command(page, "key system");
  await page.keyboard.type("Moved identity"); await page.keyboard.press("Tab"); await page.keyboard.type("/fixture/original"); await page.keyboard.press("Meta+s");
  await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  await page.getByRole("button", {name:"Relink file…",exact:true}).focus(); await page.keyboard.press("Enter");
  const dialog = page.getByRole("dialog", {name:"Relink SSH identity",exact:true});
  await expect(dialog).toContainText("Required fingerprint: SHA256:public-fixture");
  await expect(dialog.getByLabel("Storage", {exact:true})).toBeDisabled();
  await dialog.getByLabel("Private key file", {exact:true}).fill("/fixture/moved"); await page.keyboard.press("Meta+s");
  await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  await expect(page.getByText("/fixture/moved", {exact:true})).toBeVisible();
  expect(await page.evaluate(() => (window as unknown as {fixtureKeys:{keys:{id:string}[]}}).fixtureKeys.keys.map(key=>key.id))).toEqual(["key-fixture"]);
});


test("terminal requires explicit open, preserves control keys and reviews multiline paste", async ({ page }) => {
  await fixture(page);
  await command(page, "terminal open");
  await expect(page.getByRole("dialog", {name:"Open local terminal?"})).toBeVisible();
  expect(await page.evaluate(() => (window as any).fixtureCalls.filter((x: string) => x === "terminal_open_local").length)).toBe(0);
  await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  await expect(page.getByText("Connected", {exact:true})).toBeVisible();
  await expect.poll(() => page.evaluate(() => (window as any).fixtureTerminalAck.includes(1))).toBe(true);
  await page.getByRole("button", {name:"Focus terminal",exact:true}).click();
  await expect(page.locator(".xterm-helper-textarea")).toBeFocused();
  await page.keyboard.type("echo hello"); await page.keyboard.press("Control+c");
  await expect.poll(() => page.evaluate(() => String.fromCharCode(...(window as any).fixtureTerminalInput))).toContain("echo hello\x03");
  await page.evaluate(() => {
    const data = new DataTransfer(); data.setData("text/plain", "echo reviewed\n");
    document.querySelector(".xterm-helper-textarea")!.dispatchEvent(new ClipboardEvent("paste", {clipboardData:data,bubbles:true,cancelable:true}));
  });
  await expect(page.getByRole("dialog",{name:"Review terminal paste"})).toBeVisible();
  expect(await page.evaluate(() => String.fromCharCode(...(window as any).fixtureTerminalInput))).not.toContain("reviewed");
  await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  await expect.poll(() => page.evaluate(() => String.fromCharCode(...(window as any).fixtureTerminalInput))).toContain("echo reviewed");
  await page.keyboard.press("Meta+k"); await expect(page.getByRole("dialog",{name:"Command popup"})).toBeVisible();
  await page.keyboard.press("Escape");
  await page.keyboard.press("Meta+w"); await expect(page.getByRole("alertdialog")).toContainText("Terminate terminal session?");
  await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  await expect.poll(() => page.evaluate(() => (window as any).fixtureCalls.includes("terminal_close"))).toBe(true);
});

test("terminal cleanup reviews workspace changes and closes the last terminal tab", async ({ page }) => {
  await fixture(page);
  await command(page, "terminal open");
  await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
  await expect(page.getByText("Connected", {exact:true})).toBeVisible();
  await page.keyboard.press("Meta+o");
  await page.getByLabel("Workspace location", {exact:true}).fill("/fixture/other.ghosh");
  await page.getByRole("button", {name:"Open workspace",exact:true}).last().focus(); await page.keyboard.press("Enter");
  await expect(page.getByRole("alertdialog")).toContainText("Terminate terminal sessions before switching workspace?");
  await page.keyboard.press("Escape"); await page.keyboard.press("Escape");
  await page.getByRole("button", {name:"Close Overview tab",exact:true}).focus(); await page.keyboard.press("Enter");
  await expect(page.getByRole("tab")).toHaveCount(1);
  await page.getByRole("tab", {name:"Terminal",exact:true}).click();
  await page.keyboard.press("Meta+w");
  await page.getByRole("button", {name:"Terminate session",exact:true}).click();
  await expect(page.getByRole("tab", {name:"Terminal",exact:true})).toBeVisible();
  await expect(page.getByRole("button", {name:"Open local terminal",exact:true})).toBeVisible();
  await expect.poll(() => page.evaluate(() => (window as any).fixtureCalls.includes("terminal_close"))).toBe(true);
});

test("split terminals fit both panes inside the available content width", async ({ page }) => {
  await fixture(page);
  for (let i = 0; i < 2; i++) {
    await command(page, "terminal open");
    await expect(page.getByRole("dialog").getByRole("button", {name:"Cancel",exact:true})).toBeFocused();
    await page.keyboard.press("Tab"); await page.keyboard.press("Enter");
    await expect(page.getByRole("group", {name:"Terminal sessions"}).getByRole("button")).toHaveCount((i + 1)*2);
  }
  await page.keyboard.press("Meta+f");
  await page.keyboard.press("Tab"); await page.keyboard.press("Tab"); await page.keyboard.press("Space");
  await expect(page.getByRole("button", {name:"Split view",exact:true})).toHaveAttribute("aria-pressed", "true");
  await expect.poll(() => page.evaluate(() => {
    const container = document.querySelector(".terminal-panes")!.getBoundingClientRect();
    return [...document.querySelectorAll(".terminal-pane")].every(pane => {
      const rect = pane.getBoundingClientRect();
      const canvas = pane.querySelector(".xterm-screen")!.getBoundingClientRect();
      return rect.right <= container.right + 1 && canvas.right <= rect.right + 1;
    });
  })).toBe(true);
});


test("node picker connects without a second approval", async ({ page }) => {
  await fixture(page, true);
  await command(page, "node test");
  const dialog = page.getByRole("dialog", { name: "Test node connection" });
  await expect(dialog.getByLabel("Node", { exact: true })).toBeFocused();
  await dialog.getByRole("button", { name: "Connect", exact: true }).focus();
  await page.keyboard.press("Enter");
  await expect(dialog.getByRole("status")).toContainText("Authenticated successfully");
  const calls = await page.evaluate(() => (window as any).fixtureCalls);
  expect(calls.filter((cmd: string) => cmd === "node_connection_preview")).toHaveLength(1);
  expect(calls.filter((cmd: string) => cmd === "node_connection_test")).toHaveLength(1);
  await dialog.getByRole("button", { name: "Done", exact: true }).click();
  await expect(dialog).not.toBeVisible();
});

test("inventory test connects immediately including unknown hosts", async ({ page }) => {
  await fixture(page, true, true);
  await page.getByRole("navigation").getByRole("button", { name: "Nodes", exact: true }).click();
  await page.getByRole("button", { name: "Test edge1", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "Test node connection" });
  await expect(dialog.getByRole("status")).toContainText("Authenticated successfully");
  await expect(dialog.getByRole("checkbox")).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).fixtureAuthentication.allow_unknown)).toBe(true);
  expect(await page.evaluate(() => (window as any).fixtureCalls.filter((cmd: string) => cmd === "node_connection_test").length)).toBe(1);
});

test("themed selectors support keyboard commit, cancellation, typeahead and dialog focus", async ({ page }) => {
  await fixture(page);
  await command(page, "settings");
  const size = page.getByRole("combobox", { name: "Font size", exact: true });
  await size.click(); await expect(size).toBeFocused();
  await page.keyboard.press("Escape");
  await size.focus(); await page.keyboard.press("ArrowDown");
  const menu = page.getByRole("listbox", { name: "Options" });
  await expect(menu).toBeVisible();
  const row = page.getByRole("option", { name: "14px", exact: true });
  expect(await row.evaluate(element => parseFloat(getComputedStyle(element).lineHeight))).toBeGreaterThan(22);
  await page.keyboard.press("ArrowDown"); await page.keyboard.press("Escape");
  await expect(size).toBeFocused(); await expect(size).toHaveText("14px");
  await expect(menu).toBeHidden();
  await page.keyboard.press("1"); await page.keyboard.press("6"); await page.keyboard.press("Enter");
  await expect(size).toHaveText("16px");
  await size.press("Enter"); await page.keyboard.press("Tab");
  await expect(menu).toBeHidden(); await expect(page.getByLabel("Row spacing")).toBeFocused();
  await page.getByRole("button", {name:"Reset appearance"}).click();
  await command(page, "node add");
  const dialog = page.getByRole("dialog", {name:"Add node",exact:true});
  const auth = dialog.getByRole("combobox", {name:"Authentication",exact:true});
  await auth.focus(); await page.keyboard.press("Enter");
  await expect(page.getByRole("option", {name:"File key",exact:true})).toHaveAttribute("aria-disabled", "true");
  await page.keyboard.press("End"); await page.keyboard.press("Enter");
  await expect(auth).toHaveText("SSH agent");
  await auth.press("Enter"); await page.keyboard.press("Escape");
  await expect(dialog).toBeVisible(); await expect(auth).toBeFocused();
  await page.keyboard.press("Escape"); await expect(dialog).toBeHidden();
  expect(await page.locator("select").count()).toBe(0);
});

test("number shortcuts use displayed tab order and icons open actual jobs and approvals", async ({ page }) => {
  await fixture(page);
  await page.getByRole("button", { name:"Nodes",exact:true }).click();
  await page.getByRole("button", { name:"Background jobs",exact:true }).click();
  await expect(page.getByRole("heading",{name:"Background jobs",exact:true})).toBeVisible();
  await page.getByRole("button", { name:"Approvals inbox",exact:true }).click();
  await expect(page.getByRole("heading",{name:"Approvals",exact:true})).toBeVisible();
  await page.keyboard.press("Meta+2"); await expect(page.getByRole("tab",{name:"Nodes",exact:true})).toHaveAttribute("aria-selected","true");
  await page.keyboard.press("Meta+9"); await expect(page.getByRole("tab",{name:"Nodes",exact:true})).toHaveAttribute("aria-selected","true");
  await page.keyboard.press("Meta+w"); await expect(page.getByRole("tab",{name:"Nodes",exact:true})).toHaveCount(0);
  await page.keyboard.press("Meta+2"); await expect(page.getByRole("tab",{name:"Background jobs",exact:true})).toHaveAttribute("aria-selected","true");
  await page.keyboard.press("Meta+1"); await expect(page.getByLabel("Workspace name",{exact:true})).toBeVisible();
  await command(page,"keyboard");
  await expect(page.locator("dialog .shift-key").first()).toBeVisible();
});

test("node header has mixed state, filtered select-all preserves hidden selection, and controls align", async ({ page }) => {
  await fixture(page);
  await page.getByRole("button", {name:"Nodes",exact:true}).click();
  const all = page.getByRole("checkbox", {name:"Select all visible nodes",exact:true});
  await page.getByRole("checkbox", {name:"Select edge1",exact:true}).check();
  expect(await all.evaluate((element: HTMLInputElement) => element.indeterminate)).toBe(true);
  const aligned = await page.locator(".node-identity").evaluateAll(elements => elements.every(element => {
    const centers = [...element.children].map(child => { const rect = child.getBoundingClientRect(); return rect.y + rect.height / 2; });
    return Math.max(...centers) - Math.min(...centers) < 1;
  }));
  expect(aligned).toBe(true);
  await page.getByLabel("Filter nodes").fill("db1"); await all.check();
  await page.getByLabel("Filter nodes").fill("");
  await expect(page.getByRole("checkbox", {name:"Select edge1",exact:true})).toBeChecked();
  await expect(page.getByRole("checkbox", {name:"Select db1",exact:true})).toBeChecked();
  await all.uncheck(); await expect(page.getByRole("status").filter({hasText:"0 selected"})).toBeVisible();
});

test("small layouts contain text and dropdown menus at normal and 200 percent zoom", async ({ page }) => {
  await fixture(page);
  const issues = () => page.evaluate(() => {
    const errors: string[] = [];
    if (document.documentElement.scrollWidth > innerWidth) errors.push("document overflow");
    for (const element of document.querySelectorAll<HTMLElement>("button, .panel, .notice, .workspace-switch, .nav-item, input, .stat")) {
      if (!element.getClientRects().length || element.closest("[hidden], .tab-strip, .table-scroll")) continue;
      const rect = element.getBoundingClientRect();
      if (rect.right > innerWidth + 1 || rect.left < -1) errors.push(`outside viewport: ${element.textContent?.slice(0, 40)}`);
      if (element.scrollWidth > element.clientWidth + 1 && getComputedStyle(element).overflowX === "visible") errors.push(`text spills: ${element.textContent?.slice(0, 40)}`);
    }
    return errors;
  });
  for (const width of [820, 360]) {
    await page.setViewportSize({width,height:600});
    for (const zoom of [100,200]) {
      await command(page,"settings");
      await choose(page.getByLabel("Interface zoom",{exact:true}), `${zoom}%`);
      expect(await issues()).toEqual([]);
      const selector = page.getByLabel("Interface zoom",{exact:true});
      await selector.click();
      const bounds = await page.getByRole("listbox",{name:"Options"}).boundingBox();
      expect(bounds!.x).toBeGreaterThanOrEqual(0); expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(width);
      expect(bounds!.y).toBeGreaterThanOrEqual(0); expect(bounds!.y + bounds!.height).toBeLessThanOrEqual(600);
      await page.keyboard.press("Escape");
      for (const name of ["Overview","Nodes","SSH Keys","Playbooks","Tunnels","Approvals"]) {
        await page.getByRole("button",{name,exact:true}).click(); expect(await issues(), `${name} at ${width}/${zoom}`).toEqual([]);
      }
    }
  }
});

test("recent workspace command is clickable; history can be removed without closing", async ({ page }) => {
  await fixture(page);
  await page.keyboard.press("Meta+k");
  await page.getByRole("combobox", { name: "Search commands" }).fill("workspace recent");
  await page.getByRole("option", { name: /Recent workspaces/ }).click();
  const dialog = page.getByRole("dialog", { name: "Workspaces", exact: true });
  await expect(dialog).toBeVisible();
  await dialog.getByLabel("Find a workspace").fill("Other");
  await expect(dialog.getByRole("button", { name: "Remove Test fleet from history" })).toHaveCount(0);
  await dialog.getByRole("button", { name: "Remove Other fleet from history" }).click();
  await expect(dialog).toContainText("No matching workspaces");
  await dialog.getByLabel("Find a workspace").fill("");
  await dialog.getByRole("button", { name: "Clear history", exact: true }).click();
  await expect(dialog).toContainText("No recent workspaces");
  await dialog.getByRole("button", { name: "Done", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Test fleet", exact: true })).toBeVisible();
  const calls = await page.evaluate(() => (window as unknown as { fixtureCalls: string[] }).fixtureCalls);
  expect(calls).toContain("workspace_startup");
  expect(calls.filter(cmd => cmd === "workspace_forget_recent")).toHaveLength(2);
  expect(calls).not.toContain("workspace_close");
});

test("bracket shortcuts wrap tabs; close workspace guards draft then returns to Welcome", async ({ page }) => {
  await fixture(page);
  await page.getByLabel("Workspace name", { exact: true }).fill("Unsaved");
  await command(page, "settings");
  await page.keyboard.press("Meta+[");
  await expect(page.getByRole("tab", { name: /^Overview/ })).toHaveAttribute("aria-selected", "true");
  await expect(page.getByLabel("Workspace name", { exact: true })).toHaveValue("Unsaved");
  await page.keyboard.press("Meta+[");
  await expect(page.getByRole("tab", { name: "Settings", exact: true })).toHaveAttribute("aria-selected", "true");
  await page.keyboard.press("Meta+]");
  await expect(page.getByRole("tab", { name: /^Overview/ })).toHaveAttribute("aria-selected", "true");
  await page.keyboard.press("Meta+Shift+w");
  await expect(page.getByRole("alertdialog")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByLabel("Workspace name", { exact: true })).toHaveValue("Unsaved");
  await page.keyboard.press("Meta+Shift+w");
  await page.getByRole("button", { name: "Continue", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Welcome to ghosh." })).toBeVisible();
  await expect(page.getByRole("button", { name: "Remove Test fleet from history" })).toBeVisible();
  const calls = await page.evaluate(() => (window as unknown as { fixtureCalls: string[] }).fixtureCalls);
  expect(calls.filter(cmd => cmd === "workspace_close")).toHaveLength(1);
});


test("node terminal only asks for required credentials and clears its connection secret", async ({ page }) => {
  await fixture(page, true, false, true);
  await page.getByRole("navigation").getByRole("button", { name: "Nodes", exact: true }).click();
  await page.getByRole("button", { name: "Terminal edge1", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "Open node terminal" });
  await expect(dialog.getByLabel("Key passphrase", { exact: true })).toBeFocused();
  expect(await page.evaluate(() => (window as any).fixtureCalls.includes("terminal_open_node"))).toBe(false);
  await dialog.getByLabel("Key passphrase", { exact: true }).fill("disposable-ui-secret");
  await dialog.getByRole("button", { name: "Connect", exact: true }).focus();
  await page.keyboard.press("Enter");
  await expect(dialog).not.toBeVisible();
  await expect(page.getByRole("tab", { name: "Terminal", exact: true })).toHaveAttribute("aria-selected", "true");
  await expect(page.getByRole("group", { name: "Terminal sessions" })).toContainText("edge1");
  await expect(page.getByRole("status").filter({ hasText: "Connected" })).toBeVisible();
  await expect(page.locator(".xterm-helper-textarea")).toBeFocused();
  expect(await page.evaluate(() => (window as any).fixtureAuthentication.credential)).toBe("disposable-ui-secret");
  expect(await page.evaluate(() => JSON.stringify(localStorage).includes("disposable-ui-secret"))).toBe(false);
  await page.getByRole("button", { name: "Node terminal", exact: true }).click();
  await dialog.getByRole("button", { name: "Connect", exact: true }).click();
  await dialog.getByLabel("Key passphrase", { exact: true }).fill("second-disposable-secret");
  await dialog.getByRole("button", { name: "Connect", exact: true }).click();
  await expect(dialog).not.toBeVisible();
  const selectedPane = page.locator(".terminal-pane:not([hidden])");
  await expect(selectedPane.locator(".xterm-helper-textarea")).toBeFocused();
  await page.keyboard.type("pwd");
  await expect.poll(() => page.evaluate(() => (window as any).fixtureLastInputSession)).toBe("remote-fixture-2");
  await page.getByRole("button", { name: "Close session", exact: true }).click();
  await page.getByRole("button", { name: "Terminate session", exact: true }).click();
  await page.getByRole("button", { name: "Close session", exact: true }).click();
  await page.getByRole("button", { name: "Terminate session", exact: true }).click();
  await expect(page.getByText("No terminal sessions", { exact: true })).toBeVisible();
});

test("node terminal command is clickable and offers configured nodes", async ({ page }) => {
  await fixture(page);
  await page.keyboard.press("Meta+k");
  await page.getByRole("combobox", { name: "Search commands" }).fill("node terminal");
  await page.getByRole("option", { name: /Open node terminal/ }).click();
  const dialog = page.getByRole("dialog", { name: "Open node terminal" });
  await expect(dialog.getByLabel("Node", { exact: true })).toBeFocused();
  await dialog.getByRole("button", { name: "Cancel", exact: true }).click();
  expect(await page.evaluate(() => (window as any).fixtureCalls.includes("terminal_open_node"))).toBe(false);
});


test("Playbooks save syntax warnings immediately and retain unsaved drafts", async ({ page }) => {
  await fixture(page);
  await command(page, "playbook new");
  const dialog = page.getByRole("dialog", { name: "Create playbook" });
  await dialog.getByLabel("ID", { exact: true }).fill("checked-playbook");
  await expect(dialog.getByLabel("Format", {exact:true})).toHaveCount(0);
  await expect(dialog.getByRole("button", {name:"Validate",exact:true})).toHaveCount(0);
  await dialog.getByRole("button", {name:"Plain text",exact:true}).click();
  const code = "if then\n# preserve 世界";
  await dialog.getByRole("textbox", {name:"Source",exact:true}).fill(code);
  await dialog.getByLabel("Timeout (seconds; 0 = none)").fill("0");
  const runtime = await dialog.getByRole("combobox", {name:"Runtime",exact:true}).boundingBox();
  const timeout = await dialog.getByLabel("Timeout (seconds; 0 = none)").boundingBox();
  expect(Math.abs(runtime!.y-timeout!.y)).toBeLessThan(2);
  expect(Math.abs(runtime!.height-timeout!.height)).toBeLessThan(2);
  await dialog.getByRole("button", {name:"Save",exact:true}).click();
  await expect(dialog).not.toBeVisible();
  await page.getByRole("button", {name:"Edit checked-playbook",exact:true}).click();
  const edit = page.getByRole("dialog", {name:"Edit playbook"});
  await edit.getByRole("button", {name:"Plain text",exact:true}).click();
  await expect(edit.getByRole("textbox", {name:"Source",exact:true})).toHaveValue(code);
  await edit.getByRole("textbox", {name:"Source",exact:true}).fill(code+"\n# changed");
  await edit.getByRole("button", {name:"Restore Playbook",exact:true}).click();
  await edit.getByRole("button", {name:"Cancel",exact:true}).click();
  await expect(edit).toContainText("Discard this unsaved draft?");
  await edit.getByRole("button", {name:"Keep editing",exact:true}).click();
  await expect(edit.getByRole("textbox", {name:"Source",exact:true})).toHaveValue(code+"\n# changed");
});

test("Playbook ID correction keeps step code", async ({ page }) => {
  await fixture(page);
  await command(page, "playbook new");
  const dialog = page.getByRole("dialog", {name:"Create playbook"});
  const source = "#!/bin/sh\ncd ~\necho '123' > out.txt\n";
  await dialog.getByRole("button", {name:"Plain text",exact:true}).click();
  await dialog.getByRole("textbox", {name:"Source",exact:true}).fill(source);
  await dialog.getByLabel("ID", {exact:true}).fill("create a file");
  await expect(dialog.getByLabel("ID", {exact:true})).toHaveAttribute("aria-invalid","true");
  await expect(dialog.getByRole("button", {name:"Save",exact:true})).toBeDisabled();
  await dialog.getByRole("button", {name:"Use create-a-file",exact:true}).click();
  await expect(dialog.getByRole("textbox", {name:"Source",exact:true})).toHaveValue(source);
  await dialog.getByRole("button", {name:"Save",exact:true}).click();
  await page.getByRole("button", {name:"Edit create-a-file",exact:true}).click();
  await page.getByRole("dialog").getByRole("button", {name:"Plain text",exact:true}).click();
  await expect(page.getByRole("dialog").getByRole("textbox", {name:"Source",exact:true})).toHaveValue(source);
});

test("SSH agent failure identifies the target and allows retry with retained targets", async ({ page }) => {
  await fixture(page);
  await page.evaluate(() => {
    (window as any).fixtureAutomations = [{ id: "facts", kind: "notebook", source: JSON.stringify({schema:1,name:"Facts",cells:[{id:"first",title:"Facts",runtime:"shell",code:"uname -a",timeout_s:0,success_check:null}]}) }];
    (window as any).fixtureConnectionError = { node: "db1", error: { code: "agent_unavailable", message: "The SSH agent is unavailable. Load a key or choose File key authentication." } };
  });
  await page.getByRole("navigation").getByRole("button", { name: "Playbooks", exact: true }).click();
  await page.getByRole("button", { name: "Run facts", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: /^Run / });
  await dialog.getByRole("checkbox", { name: /db1/ }).check();
  await dialog.getByRole("button", { name: "Start run", exact: true }).click();
  await expect(dialog.getByRole("alert")).toContainText("db1: The SSH agent is unavailable");
  await expect(dialog.getByRole("checkbox", { name: /edge1/ })).toBeChecked();
  await expect(dialog.getByRole("checkbox", { name: /db1/ })).toBeChecked();
  expect(await page.evaluate(() => (window as any).fixtureCalls.includes("run_start"))).toBe(false);
  await page.evaluate(() => { (window as any).fixtureConnectionError = null; });
  await dialog.getByRole("button", { name: "Start run", exact: true }).click();
  await expect(dialog).not.toBeVisible();
  const request = await page.evaluate(() => (window as any).fixtureRunRequest);
  expect(request.targets.map((target: any) => target.node_id)).toEqual(["edge1", "db1"]);
  expect(request.targets.every((target: any) => target.connection.allow_unknown === true)).toBeTruthy();
});

test("inventory terminal opens directly once without approval", async ({ page }) => {
  await fixture(page);
  await page.getByRole("navigation").getByRole("button", { name: "Nodes", exact: true }).click();
  await page.getByRole("button", { name: "Terminal edge1", exact: true }).click();
  await expect(page.getByRole("tab", { name: "Terminal", exact: true })).toHaveAttribute("aria-selected", "true");
  await expect(page.locator(".xterm-helper-textarea")).toBeFocused();
  expect(await page.evaluate(() => (window as any).fixtureTerminalCount)).toBe(1);
});

test("saved multistep Playbook runs on explicit targets and can stop", async ({ page }) => {
  await fixture(page);
  await command(page, "playbook new");
  await page.getByRole("dialog").getByRole("button", {name:"Save",exact:true}).click();
  await page.getByRole("button", {name:"Run multi-step",exact:true}).click();
  const dialog=page.getByRole("dialog", {name:/^Run /});
  await dialog.getByRole("checkbox", {name:/db1/}).check();
  await dialog.getByRole("button", {name:"Start run",exact:true}).click();
  await expect(dialog).not.toBeVisible();
  const request = await page.evaluate(() => (window as any).fixtureRunRequest);
  expect(request.targets.map((target:any)=>target.node_id)).toEqual(["edge1","db1"]);
  expect(request.mode).toBe("apply");
  await page.getByRole("button", {name:"Stop run",exact:true}).click();
  await expect(page.getByText("outcome unknown", {exact:true}).first()).toBeVisible();
});

test("RU physical shortcuts navigate, search, save and close; composition and terminal Ctrl are preserved", async ({ page }) => {
  await fixture(page);
  const key = (key: string, code: string, modifiers: Record<string, boolean> = {}) => page.evaluate(({key, code, modifiers}) => {
    const event = new KeyboardEvent("keydown", { key, code, metaKey: true, bubbles: true, cancelable: true, ...modifiers });
    window.dispatchEvent(event); return event.defaultPrevented;
  }, {key, code, modifiers});
  expect(await key("л", "KeyK", {isComposing: true})).toBe(false);
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await key("л", "KeyK");
  await expect(page.getByRole("combobox", {name: "Search commands"})).toBeFocused();
  await page.keyboard.press("Escape");
  await key("З", "KeyP", {shiftKey: true});
  await expect(page.getByRole("dialog", {name: "Command popup"})).toBeVisible();
  await page.keyboard.press("Escape");
  await page.getByLabel("Workspace name", {exact: true}).fill("RU saved");
  await key("ы", "KeyS");
  await expect(page.getByRole("heading", {name: "RU saved", exact: true})).toBeVisible();
  await page.getByRole("navigation").getByRole("button", {name: "Nodes", exact: true}).click();
  await key("а", "KeyF");
  await expect(page.getByLabel("Filter nodes")).toBeFocused();
  await key("т", "KeyN");
  await expect(page.getByRole("dialog", {name: "Add node", exact: true})).toBeVisible();
  await page.keyboard.press("Escape");
  await key("х", "BracketLeft");
  await expect(page.getByRole("tab", {name: "Overview", exact: true})).toHaveAttribute("aria-selected", "true");
  await key("ъ", "BracketRight");
  await expect(page.getByRole("tab", {name: "Nodes", exact: true})).toHaveAttribute("aria-selected", "true");
  await key("!", "Digit1");
  await expect(page.getByRole("tab", {name: "Overview", exact: true})).toHaveAttribute("aria-selected", "true");
  await key("2", "Digit2");
  await key("ц", "KeyW");
  await expect(page.getByRole("tab", {name: "Nodes", exact: true})).toHaveCount(0);
  await key("щ", "KeyO");
  await expect(page.getByRole("dialog").getByLabel("Workspace location", {exact: true})).toBeVisible();
  await page.keyboard.press("Escape");
  await command(page, "terminal open");
  await page.getByRole("button", {name: "Open shell", exact: true}).click();
  await expect(page.locator(".xterm-helper-textarea")).toBeFocused();
  expect(await key("л", "KeyK", {metaKey: false, ctrlKey: true})).toBe(false);
  await expect(page.getByRole("dialog")).toHaveCount(0);
});

test("legacy Ansible history remains exportable while Playbooks hides old formats", async ({page}) => {
  await fixture(page);
  await page.evaluate(()=>{
    (window as any).fixtureAutomations=[{id:"existing-project",kind:"ansible",source:"schema: 1"}];
    (window as any).fixtureRuns=[{id:"run-fixture",job_id:"old",workspace_id:"fixture",revision:"r1",automation_id:"existing-project",kind:"ansible",mode:"check",targets:["edge1"],state:"succeeded",started_at:Date.now(),finished_at:Date.now(),results:[],message:"Check complete"}];
  });
  await page.getByRole("navigation").getByRole("button", {name:"Playbooks",exact:true}).click();
  await expect(page.getByRole("button", {name:"Run existing-project",exact:true})).toHaveCount(0);
  await page.getByRole("navigation").getByRole("button", {name:"Runs",exact:true}).click();
  const download=page.waitForEvent("download");
  await page.getByRole("button", {name:"Export results · JSON"}).click();
  expect((await download).suggestedFilename()).toContain("run-fixture");
});

test("ComfyUI recipe enforces pins and hashes then saves ordinary Ansible YAML without overflow", async ({page}) => {
  await fixture(page);
  await command(page, "recipe apply");
  const dialog = page.getByRole("dialog", {name: "Create ComfyUI recipe"});
  await page.screenshot({path: "docs/evidence/ui-shell/34-comfy-wizard-fixture.png"});
  await dialog.getByRole("button", {name: "Review Ansible YAML", exact: true}).click();
  await expect(dialog.getByRole("alert")).toContainText("Add at least one");
  await dialog.getByRole("button", {name: "Add model", exact: true}).click();
  await dialog.getByLabel("Public HTTPS model URL").fill("https://example.com/model.safetensors");
  await dialog.getByLabel("Path inside models/ · e.g. loras/example.safetensors").fill("loras/example.safetensors");
  await dialog.getByRole("button", {name: "Review Ansible YAML", exact: true}).click();
  await expect(dialog.getByRole("alert")).toContainText("SHA-256");
  await dialog.getByLabel("SHA-256 · 64 hex characters").fill("a".repeat(64));
  await dialog.getByRole("button", {name: "Add plugin", exact: true}).click();
  await dialog.getByLabel("Folder name").fill("example-node");
  await dialog.getByLabel("Public HTTPS Git repository").fill("https://github.com/example/example-node.git");
  await dialog.getByLabel("Full Git commit · 40 hex characters").fill("b".repeat(40));
  await page.setViewportSize({width: 460, height: 760});
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({path: "docs/evidence/ui-shell/33-comfy-recipe-fixture.png"});
  await dialog.getByRole("button", {name: "Review Ansible YAML", exact: true}).click();
  const editor = page.getByRole("dialog", {name: "Create playbook"});
  await editor.getByRole("button", { name: "Plain text", exact: true }).click();
  const source = await editor.getByLabel("Source", {exact: true}).inputValue();
  expect(source).toContain("hosts: all"); expect(source).toContain("ansible.builtin.git:");
  expect(source).toContain("ansible.builtin.get_url:"); expect(source).not.toContain("schema: 1");
  // Save the exact generated YAML for an independent upstream syntax check.
  await import("node:fs/promises").then(fs => fs.writeFile("/tmp/ghosh-comfy-generated.yml", source));
  await editor.getByRole("button", {name: "Save", exact: true}).click();
  await expect(page.getByRole("button", {name: "Run comfy-runpod", exact: true})).toBeVisible();
  await page.getByRole("button", {name: "Edit comfy-runpod", exact: true}).click();
  await page.getByRole("dialog").getByRole("button", { name: "Plain text", exact: true }).click();
  await expect(page.getByRole("dialog").getByLabel("Source", {exact: true})).toHaveValue(source);
});

test("offline code editor highlights, completes, expands and saves with RU physical shortcut", async ({ page }) => {
  await fixture(page);
  await command(page, "playbook new");
  const dialog = page.getByRole("dialog", { name: "Create playbook" });
  await expect(dialog.locator(".monaco-editor")).toBeVisible();
  await expect(dialog.locator(".view-line span[class*=mtk]").first()).toBeVisible();
  await expect(dialog).toHaveClass(/editor-expanded/);
  const input = dialog.getByRole("textbox", { name: "Source code editor" });
  await input.focus(); await page.keyboard.press("Meta+a");
  await page.keyboard.insertText("if then\n");
  await expect(dialog.locator(".editor-diagnostics")).toContainText("Expected condition");
  await page.keyboard.press("Meta+a"); await page.keyboard.insertText("if");
  await page.keyboard.press("Control+Space");
  await expect(dialog.locator(".suggest-widget.visible")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveClass(/editor-expanded/);
  await page.keyboard.press("Escape");
  await expect(dialog).not.toHaveClass(/editor-expanded/);
  await input.focus(); await page.keyboard.press("Meta+a");
  const source = "#!/bin/sh\nprintf 'editor exact source 世界\\n'\n";
  await page.keyboard.insertText(source);
  await page.setViewportSize({width: 460, height: 760});
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({path: "docs/evidence/ui-shell/35-code-editor-fixture.png"});
  await input.dispatchEvent("keydown", {key:"ы",code:"KeyS",metaKey:true,bubbles:true});
  await expect(dialog).not.toBeVisible();
  expect(await page.evaluate(() => JSON.parse((window as any).fixtureAutomations[0].source).cells[0].code)).toBe(source);
});

test("live run progress and logs survive navigation and compact layout", async ({ page }) => {
  await fixture(page);
  await page.evaluate(() => {
    (window as any).fixtureRuns = [{id:"live-test",job_id:"job-live",workspace_id:"fixture",revision:"r1",automation_id:"multi-step-file",kind:"playbook",mode:"apply",targets:["edge1","db1"],state:"running",started_at:Date.now(),finished_at:null,results:[],message:"Targets execute sequentially.",progress:{total_steps:6,completed_steps:1,node_id:"edge1",step:"Write and verify file",phase:"apply",output:"Creating directory\nWriting file\n",truncated:false}}];
  });
  await expect(page.getByRole("button", {name:"Running multi-step-file. Open live logs"})).toContainText("Running step 2 of 6");
  await page.getByRole("button", {name:"Running multi-step-file. Open live logs"}).click();
  await expect(page.getByLabel("Compact live output")).toContainText("Writing file");
  await page.getByRole("button", {name:"Open full run",exact:true}).click();
  await expect(page.getByLabel("multi-step-file overall progress")).not.toHaveAttribute("value");
  await page.getByRole("navigation").getByRole("button", {name:"Nodes",exact:true}).click();
  await page.evaluate(() => { const run = (window as any).fixtureRuns[0]; run.progress.completed_steps=4;run.progress.node_id="db1";run.progress.output="Second node still running\n"; });
  await expect(page.getByRole("button", {name:"Running multi-step-file. Open live logs"})).toContainText("Running step 5 of 6");
  await page.getByRole("button", {name:"Running multi-step-file. Open live logs"}).click();
  await expect(page.getByLabel("Compact live output")).toContainText("Second node still running");
  await page.getByRole("button", {name:"Open full run",exact:true}).click();
  await page.setViewportSize({width:460,height:760});
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({path:"docs/evidence/ui-shell/36-live-run-fixture.png"});
  await page.evaluate(() => { const run = (window as any).fixtureRuns[0];run.state="succeeded";run.progress.completed_steps=6;run.progress.output=""; });
  await expect(page.getByLabel("multi-step-file overall progress")).toHaveAttribute("value","100");
  await expect(page.getByRole("button", {name:"Finished multi-step-file. Open live logs"})).toContainText("Finished");
});


test("new file-identity nodes inherit global system keys and retain explicit agent choice", async ({ page }) => {
  await fixture(page, true, false, false, 2);
  await page.evaluate(() => {
    (window as any).fixtureKeys = {revision:"keys-2",default_key:"system-key",keys:[{id:"system-key",label:"System key",path:"/fixture/.ssh/id_ed25519",storage:"reference",public:{algorithm:"ssh-ed25519",fingerprint:"SHA256:system-fixture"},encrypted:false,status:"available"}]};
  });
  await command(page, "node add");
  const dialog = page.getByRole("dialog", {name:"Add node",exact:true});
  await expect(dialog.getByLabel("Authentication", {exact:true})).toHaveText("System keys (default)");
  await choose(dialog.getByLabel("Authentication", {exact:true}), "SSH agent");
  await expect(dialog.getByLabel("SSH agent socket (optional)")).toBeVisible();
  await expect(dialog.getByLabel("SSH identity", {exact:true})).toHaveCount(0);
  await choose(dialog.getByLabel("Authentication", {exact:true}), "File key");
  await expect(dialog.getByText("Global: SHA256:system-fixture · available", {exact:true})).toBeVisible();
});

test("Playbook authoring maximizes, drags stable steps and runs a saved cell with inline output", async ({page}) => {
  await fixture(page);
  await page.getByRole("navigation").getByRole("button", {name:"Playbooks",exact:true}).click();
  await page.getByRole("button", {name:"New playbook",exact:true}).click();
  const dialog=page.getByRole("dialog");
  await expect(dialog.getByLabel("Playbook name", {exact:true})).toHaveValue("Multistep Playbook");
  await dialog.getByRole("button", {name:"Add step",exact:true}).click();
  const cells=dialog.getByRole("article");
  await cells.nth(1).getByLabel("Step title", {exact:true}).fill("Inspect Python");
  await choose(cells.nth(1).getByRole("combobox", {name:"Runtime",exact:true}), "Python");
  await cells.nth(1).getByRole("button", {name:"Plain text",exact:true}).click();
  await cells.nth(1).getByRole("textbox", {name:"Source",exact:true}).fill("import platform; print(platform.system())");
  await expect(dialog).toHaveClass(/editor-expanded/);
  await cells.nth(1).getByRole("button", {name:"Expand step",exact:true}).click();
  await expect(cells.nth(1)).toHaveClass(/step-maximized/);
  await cells.nth(1).getByRole("button", {name:"Restore step",exact:true}).click();
  // Keep both handles visible so browser auto-scrolling does not move the source
  // between Playwright's pointer positioning and the drag gesture.
  await page.setViewportSize({width:1160,height:1800});
  await cells.nth(1).getByRole("button", {name:/Drag step 2/}).dragTo(cells.nth(0).getByRole("button", {name:/Drag step 1/}));
  await expect(cells.nth(0).getByLabel("Step title", {exact:true})).toHaveValue("Inspect Python");
  await page.setViewportSize({width:1160,height:780});
  await cells.nth(0).getByLabel("Timeout (seconds; 0 = none)").fill("0");
  await cells.nth(1).getByRole("textbox", {name:"Source code editor"}).focus();
  await page.keyboard.press("Meta+s");
  await expect(dialog).not.toBeVisible();
  await page.getByRole("button", {name:"Open Playbook multi-step",exact:true}).click();
  await expect(page.getByRole("heading",{name:"Multistep Playbook",exact:true})).toBeVisible();
  const saved=await page.evaluate(()=>JSON.parse((window as any).fixtureAutomations[0].source));
  expect(saved.cells[0].timeout_s).toBe(0); expect(saved.cells[0].runtime).toBe("python"); expect(saved.cells[1].id).toBe("step-1");
  await page.getByRole("button",{name:"Run step 2",exact:true}).click();
  await expect(page.getByRole("dialog").getByRole("combobox", {name:"Steps to run",exact:true})).toHaveText("First step");
  await page.getByRole("dialog").getByRole("checkbox",{name:/edge1/}).uncheck();
  await page.getByRole("dialog").getByRole("checkbox",{name:/Local machine/}).check();
  await page.getByRole("button",{name:"Start run",exact:true}).click();
  expect(await page.evaluate(()=>(window as any).fixtureRunRequest.cells)).toEqual(["step-1"]);
  await page.evaluate(()=>{(window as any).fixtureRuns[0].results=[{node_id:"@local",name:"step-1 · First step",phase:"cell",state:"changed",output:"Hello from step 1",exit_code:0,truncated:false}];});
  await page.getByRole("navigation").getByRole("button",{name:"Playbooks",exact:true}).click();
  await expect(page.getByRole("article",{name:"Step 2: First step"}).getByText("Hello from step 1",{exact:true})).toBeVisible();
  await page.getByRole("button",{name:"Minimize run monitor",exact:true}).click();
  await page.screenshot({path:"docs/evidence/ui-shell/40-notebook-fixture.png"});
  await command(page,"settings");
  await choose(page.getByLabel("Interface zoom",{exact:true}), "200%");
  await page.setViewportSize({width:460,height:760});
  await page.getByRole("navigation").getByRole("button",{name:"Playbooks",exact:true}).click();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth <= innerWidth)).toBe(true);
});

test("Runs preserves legacy drift history and only offers execution history actions", async ({page}) => {
  await fixture(page);
  await page.evaluate(()=>{
    (window as any).fixtureAutomations=[{id:"ms-script",kind:"playbook",source:"schema: 1"}];
    (window as any).fixtureRuns=[{id:"finished",job_id:"old",workspace_id:"fixture",revision:"r1",automation_id:"ms-script",kind:"playbook",mode:"check",targets:["@local"],state:"succeeded",started_at:Date.now(),finished_at:Date.now(),results:[{node_id:"@local",name:"Create",phase:"check",state:"drift",output:"",exit_code:10,truncated:false}],message:"Check complete"}];
  });
  await page.getByRole("navigation").getByRole("button",{name:"Runs",exact:true}).click();
  await expect(page.getByText("Check complete · changes needed",{exact:true})).toBeVisible();
  await expect(page.getByRole("button",{name:"Apply current saved playbook",exact:true})).toHaveCount(0);
  await page.getByRole("button",{name:"Clear finished runs",exact:true}).click();
  await expect(page.getByRole("dialog")).toContainText("including records beyond the latest 100");
  await page.getByRole("button",{name:"Remove history",exact:true}).click();
  await expect(page.getByText("No runs yet",{exact:true})).toBeVisible();
});

test("system authentication requires no separate setup wizard", async ({page}) => {
  await fixture(page);
  await page.getByRole("navigation").getByRole("button",{name:"SSH Keys",exact:true}).click();
  await expect(page.getByRole("button",{name:"Connect nodes with system keys…",exact:true})).toHaveCount(0);
  await command(page,"node add");
  await expect(page.getByLabel("Authentication",{exact:true})).toHaveText("System keys (default)");
  expect(await page.evaluate(()=>(window as any).fixtureCalls.includes("keys_commit"))).toBe(false);
});

test("compact monitor leaves an independent SSH terminal usable during a run", async ({page}) => {
  await fixture(page);
  await page.evaluate(()=>{
    (window as any).fixtureRuns=[{id:"active",job_id:"bg",workspace_id:"fixture",revision:"r1",automation_id:"notebook",kind:"notebook",mode:"apply",targets:["edge1"],state:"running",started_at:Date.now(),finished_at:null,results:[],message:"Running",progress:{total_steps:2,completed_steps:0,node_id:"edge1",step:"first · Inspect",phase:"cell",output:"streaming",truncated:false}}];
    (window as any).fixtureOperations.jobs.jobs.push({id:"bg",kind:"automation_run",state:"running",cancellable:true,started_at:Date.now(),finished_at:null,error:null,stages:[]});
  });
  await expect(page.getByRole("complementary",{name:"Live run monitor"})).toBeVisible();
  await command(page,"node terminal");
  const dialog=page.getByRole("dialog");
  await dialog.getByRole("button",{name:"Connect",exact:true}).click();
  await expect(page.locator(".xterm-helper-textarea")).toBeFocused();
  await page.keyboard.type("echo still-usable");
  await expect.poll(()=>page.evaluate(()=>(window as any).fixtureTerminalInput.length)).toBeGreaterThan(0);
  await expect(page.getByLabel("Compact live output")).toContainText("streaming");
  await expect(page.getByRole("complementary",{name:"Live run monitor"})).toContainText("Running step 1 of 2");
  await expect(page.getByRole("complementary",{name:"Live run monitor"}).getByRole("progressbar")).not.toHaveAttribute("value");
  await page.screenshot({path:"docs/evidence/ui-shell/41-background-monitor-fixture.png"});
  await page.getByRole("button",{name:"Minimize run monitor",exact:true}).click();
  await command(page,"settings");
  await choose(page.getByLabel("Interface zoom",{exact:true}), "200%");
  await page.setViewportSize({width:460,height:760});
  expect(await page.evaluate(()=>document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.getByRole("button",{name:"Running notebook. Open live logs",exact:true}).click();
  const bounds=await page.getByRole("complementary",{name:"Live run monitor"}).boundingBox();
  expect(bounds!.x).toBeGreaterThanOrEqual(0); expect(bounds!.x+bounds!.width).toBeLessThanOrEqual(461);
  await page.getByRole("button",{name:"Minimize run monitor",exact:true}).click();
  await expect(page.getByRole("complementary",{name:"Live run monitor"})).not.toBeVisible();
});

test("Playbook file drop is inert, saves exact source and exports all steps", async ({page}) => {
  await fixture(page);
  await page.getByRole("navigation").getByRole("button",{name:"Playbooks",exact:true}).click();
  const source=JSON.stringify({schema:1,name:"Imported playbook",cells:[{id:"keep-id",title:"No timeout",runtime:"shell",code:"echo imported",timeout_s:-1,success_check:"true"}]},null,2);
  const data=await page.evaluateHandle(source=>{ const transfer=new DataTransfer(); transfer.items.add(new File([source],"imported.steps.yml",{type:"text/yaml"})); return transfer; },source);
  await page.getByRole("region",{name:"Import Playbook file"}).dispatchEvent("drop",{dataTransfer:data});
  const dialog=page.getByRole("dialog",{name:"Create playbook"});
  await expect(dialog.getByLabel("Playbook name",{exact:true})).toHaveValue("Imported playbook");
  expect(await page.evaluate(()=>(window as any).fixtureAutomations.length)).toBe(0);
  await dialog.getByRole("button",{name:"Save",exact:true}).click();
  await expect(dialog).not.toBeVisible();
  await page.getByRole("button",{name:"Export imported",exact:true}).click();
  await expect.poll(()=>page.evaluate(()=>(window as any).fixtureExport?.source)).toBe(source);
  expect(await page.evaluate(()=>(window as any).fixtureCalls.includes("run_start"))).toBe(false);
  await page.evaluate(source=>{(window as any).fixtureImport={id:"picked",kind:"notebook",source};},source);
  await page.getByRole("button",{name:"Import Playbook",exact:true}).click();
  await expect(dialog.getByLabel("ID",{exact:true})).toHaveValue("picked");
});


test("Monitor explicitly connects, streams bounded output and pauses hidden resource probes", async ({page})=>{
  await fixture(page);
  await page.getByRole("navigation").getByRole("button",{name:"Monitor",exact:true}).click();
  await expect(page.getByRole("navigation").getByRole("button",{name:"Scripts",exact:true})).toHaveCount(0);
  expect(await page.evaluate(()=>(window as any).fixtureCalls.includes("monitor_open"))).toBe(false);
  await page.getByRole("button",{name:"Connect Monitor",exact:true}).click();
  await expect(page.getByLabel("Node resource consumption")).toContainText("12.5%");
  expect(await page.evaluate(()=>(window as any).fixtureCalls.includes("monitor_start"))).toBe(false);
  await page.getByLabel("Log command",{exact:true}).fill("journalctl -f --no-pager");
  await page.getByRole("button",{name:"Run log command",exact:true}).click();
  await expect(page.getByLabel("Monitor output")).toContainText("monitor live fixture");
  await page.getByRole("navigation").getByRole("button",{name:"Nodes",exact:true}).click();
  await expect.poll(()=>page.evaluate(()=>(window as any).fixtureMonitorVisible)).toBe(false);
  await page.getByRole("navigation").getByRole("button",{name:"Monitor",exact:true}).click();
  await expect.poll(()=>page.evaluate(()=>(window as any).fixtureMonitorVisible)).toBe(true);
  await page.getByRole("button",{name:"Stop",exact:true}).click();
  await expect(page.getByRole("button",{name:"Connect Monitor",exact:true})).toBeVisible();
  expect(await page.evaluate(()=>(window as any).fixtureCalls.includes("run_start"))).toBe(false);
});

test("restoring example Playbooks preserves every existing ID",async({page})=>{
  await fixture(page);
  await page.evaluate(()=>{(window as any).fixtureAutomations=[{id:"steps-success",kind:"notebook",source:JSON.stringify({schema:1,name:"My edited example",cells:[{id:"first",title:"Mine",runtime:"shell",code:"echo mine",timeout_s:0,success_check:null}]})}];});
  await page.getByRole("navigation").getByRole("button",{name:"Playbooks",exact:true}).click();
  await page.getByRole("button",{name:"Restore example Playbooks",exact:true}).click();
  await expect(page.getByRole("status").filter({hasText:"Restored: steps-failure, steps-timeout"})).toBeVisible();
  const read=()=>page.evaluate(()=>(window as any).fixtureAutomations);
  const before=await read();expect(before).toHaveLength(3);expect(JSON.parse(before[0].source).name).toBe("My edited example");
  await page.getByRole("button",{name:"Restore example Playbooks",exact:true}).click();
  await expect(page.getByRole("status").filter({hasText:"Restored: none"})).toBeVisible();
  expect(await read()).toEqual(before);
});

test("shortcut overlay releases and terminal session navigation wraps with guarded close",async({page})=>{
  await fixture(page);
  await page.keyboard.down("Meta"); await page.keyboard.down("Alt");
  await expect(page.getByRole("region",{name:"Keyboard shortcut overlay"})).toBeVisible();
  await page.keyboard.up("Alt");await page.keyboard.up("Meta");
  await expect(page.getByRole("region",{name:"Keyboard shortcut overlay"})).not.toBeVisible();
  for(let i=0;i<2;i++){await command(page,"terminal open");await page.getByRole("button",{name:"Open shell",exact:true}).click();}
  const tabs=page.getByRole("group",{name:"Terminal sessions"});
  await page.keyboard.press("Meta+Shift+[");
  await expect(tabs.getByRole("button",{name:"Local 1",exact:true})).toHaveAttribute("aria-pressed","true");
  await page.keyboard.press("Meta+Shift+[");
  await expect(tabs.getByRole("button",{name:"Local 2",exact:true})).toHaveAttribute("aria-pressed","true");
  await page.keyboard.press("Meta+w");
  await expect(page.getByRole("alertdialog")).toContainText("Terminate terminal session?");
  await page.getByRole("button",{name:"Terminate session",exact:true}).click();
  await expect(tabs.getByRole("button",{name:"Close terminal Local machine",exact:true})).toHaveCount(1);
  await tabs.getByRole("button",{name:"Close terminal Local machine",exact:true}).click();
  await page.getByRole("button",{name:"Terminate session",exact:true}).click();
  await expect(page.getByRole("tab",{name:"Terminal",exact:true})).toBeVisible();
  await expect(page.getByRole("button",{name:"Open local terminal",exact:true})).toBeVisible();
});
