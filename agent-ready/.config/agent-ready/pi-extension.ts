import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { execFile } from "child_process";
import { homedir } from "os";
import { join } from "path";

const agentReady = join(homedir(), "dev", "dotfiles", "scripts", "agent-ready");

function run(action: "set" | "clear") {
	if (!process.env.TMUX_PANE) return;
	const args = action === "set" ? ["set", "--source", "pi"] : ["clear"];
	execFile(agentReady, args, () => {});
}

export default function (pi: ExtensionAPI) {
	pi.on("input", async () => run("clear"));
	pi.on("agent_start", async () => run("clear"));
	pi.on("agent_end", async () => run("set"));
	pi.on("session_shutdown", async () => run("clear"));
}
