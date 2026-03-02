// ── Nodes page ──────────────────────────────────────────────

import { signal } from "@preact/signals";
import { html } from "htm/preact";
import { render } from "preact";
import { useEffect } from "preact/hooks";
import { onEvent } from "./events.js";
import { sendRpc } from "./helpers.js";
import { ConfirmDialog, requestConfirm } from "./ui.js";

// ── Signals ─────────────────────────────────────────────────
var nodes = signal([]);
var pendingPairs = signal([]);
var pairedDevices = signal([]);
var loading = signal(false);
var activeTab = signal("connected"); // "connected" | "paired" | "pending"
var toasts = signal([]);
var toastId = 0;

// ── Helpers ─────────────────────────────────────────────────
function showToast(message, type) {
	var id = ++toastId;
	toasts.value = toasts.value.concat([{ id: id, message: message, type: type }]);
	setTimeout(() => {
		toasts.value = toasts.value.filter((t) => t.id !== id);
	}, 4000);
}

async function refreshNodes() {
	loading.value = true;
	try {
		var res = await sendRpc("node.list", {});
		if (res?.ok) nodes.value = res.payload || [];
	} catch {
		// ignore
	}
	loading.value = false;
}

async function refreshPendingPairs() {
	try {
		var res = await sendRpc("node.pair.list", {});
		if (res?.ok) pendingPairs.value = res.payload || [];
	} catch {
		// ignore
	}
}

async function refreshPairedDevices() {
	try {
		var res = await sendRpc("device.pair.list", {});
		if (res?.ok) pairedDevices.value = res.payload || [];
	} catch {
		// ignore
	}
}

async function refreshAll() {
	await Promise.all([refreshNodes(), refreshPendingPairs(), refreshPairedDevices()]);
}

async function approvePair(id) {
	var res = await sendRpc("node.pair.approve", { id });
	if (res?.ok) {
		showToast("Pairing approved — device token issued", "success");
		await refreshAll();
	} else {
		showToast(res?.error?.message || "Failed to approve", "error");
	}
}

async function rejectPair(id) {
	var res = await sendRpc("node.pair.reject", { id });
	if (res?.ok) {
		showToast("Pairing rejected", "success");
		await refreshAll();
	} else {
		showToast(res?.error?.message || "Failed to reject", "error");
	}
}

async function revokeDevice(deviceId) {
	var ok = await requestConfirm(
		`Revoke device "${deviceId}"?`,
		"This will disconnect the device and invalidate its token.",
	);
	if (!ok) return;
	var res = await sendRpc("device.token.revoke", { deviceId });
	if (res?.ok) {
		showToast("Device token revoked", "success");
		await refreshAll();
	} else {
		showToast(res?.error?.message || "Failed to revoke", "error");
	}
}

// ── Components ──────────────────────────────────────────────

function TabBar() {
	var tabs = [
		{ id: "connected", label: "Connected", count: nodes.value.length },
		{ id: "paired", label: "Paired Devices", count: pairedDevices.value.length },
		{ id: "pending", label: "Pending", count: pendingPairs.value.length },
	];

	return html`<div class="flex gap-1 mb-4">
		${tabs.map(
			(t) =>
				html`<button
					key=${t.id}
					class="px-3 py-1.5 text-sm rounded-md transition-colors ${
						activeTab.value === t.id
							? "bg-[var(--accent)] text-white"
							: "bg-[var(--surface-alt)] text-[var(--text-muted)] hover:bg-[var(--hover)]"
					}"
					onClick=${() => (activeTab.value = t.id)}
				>
					${t.label}${t.count > 0 ? html` <span class="ml-1 opacity-70">(${t.count})</span>` : null}
				</button>`,
		)}
	</div>`;
}

function ConnectedNodesList() {
	if (nodes.value.length === 0) {
		return html`<div class="text-sm text-[var(--text-muted)] py-8 text-center">
			<p class="mb-2">No nodes connected.</p>
			<p class="text-xs">
				Run <code class="bg-[var(--surface-alt)] px-1 py-0.5 rounded">moltis node run --host ws://... --token ...</code> on a remote machine to connect.
			</p>
		</div>`;
	}

	return html`<div class="flex flex-col gap-2">
		${nodes.value.map(
			(n) =>
				html`<div
					key=${n.nodeId}
					class="flex items-center gap-3 p-3 rounded-lg bg-[var(--surface-alt)] border border-[var(--border)]"
				>
					<div class="w-2 h-2 rounded-full bg-green-500 shrink-0" title="Connected"></div>
					<div class="flex-1 min-w-0">
						<div class="text-sm font-medium text-[var(--text-strong)] truncate">
							${n.displayName || n.nodeId}
						</div>
						<div class="text-xs text-[var(--text-muted)]">
							${n.platform || "unknown"} · v${n.version || "?"}
							${n.remoteIp ? html` · ${n.remoteIp}` : null}
						</div>
						${
							n.capabilities?.length
								? html`<div class="text-xs text-[var(--text-muted)] mt-1">
									caps: ${n.capabilities.join(", ")}
								</div>`
								: null
						}
					</div>
				</div>`,
		)}
	</div>`;
}

function PairedDevicesList() {
	if (pairedDevices.value.length === 0) {
		return html`<div class="text-sm text-[var(--text-muted)] py-8 text-center">
			No paired devices.
		</div>`;
	}

	return html`<div class="flex flex-col gap-2">
		${pairedDevices.value.map(
			(d) =>
				html`<div
					key=${d.deviceId}
					class="flex items-center gap-3 p-3 rounded-lg bg-[var(--surface-alt)] border border-[var(--border)]"
				>
					<div class="flex-1 min-w-0">
						<div class="text-sm font-medium text-[var(--text-strong)] truncate">
							${d.displayName || d.deviceId}
						</div>
						<div class="text-xs text-[var(--text-muted)]">
							${d.platform || "unknown"}
							${d.createdAt ? html` · paired ${d.createdAt}` : null}
						</div>
					</div>
					<button
						class="provider-btn-danger text-xs px-2 py-1"
						onClick=${() => revokeDevice(d.deviceId)}
					>
						Revoke
					</button>
				</div>`,
		)}
	</div>`;
}

function PendingPairsList() {
	if (pendingPairs.value.length === 0) {
		return html`<div class="text-sm text-[var(--text-muted)] py-8 text-center">
			No pending pairing requests.
		</div>`;
	}

	return html`<div class="flex flex-col gap-2">
		${pendingPairs.value.map(
			(r) =>
				html`<div
					key=${r.id}
					class="flex items-center gap-3 p-3 rounded-lg bg-[var(--surface-alt)] border border-[var(--border)]"
				>
					<div class="flex-1 min-w-0">
						<div class="text-sm font-medium text-[var(--text-strong)] truncate">
							${r.displayName || r.deviceId}
						</div>
						<div class="text-xs text-[var(--text-muted)]">${r.platform || "unknown"}</div>
					</div>
					<div class="flex gap-1.5">
						<button
							class="provider-btn text-xs px-2 py-1"
							onClick=${() => approvePair(r.id)}
						>
							Approve
						</button>
						<button
							class="provider-btn-secondary text-xs px-2 py-1"
							onClick=${() => rejectPair(r.id)}
						>
							Reject
						</button>
					</div>
				</div>`,
		)}
	</div>`;
}

function Toasts() {
	if (toasts.value.length === 0) return null;
	return html`<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
		${toasts.value.map(
			(t) =>
				html`<div
					key=${t.id}
					class="px-4 py-2 rounded-lg text-sm shadow-lg ${
						t.type === "error" ? "bg-red-600 text-white" : "bg-green-600 text-white"
					}"
				>
					${t.message}
				</div>`,
		)}
	</div>`;
}

// ── Main component ──────────────────────────────────────────

function NodesPage() {
	useEffect(() => {
		refreshAll();

		// Subscribe to presence events for live updates.
		var unsub = onEvent("presence", () => {
			refreshNodes();
		});
		var unsubPair = onEvent("node.pair.requested", () => {
			refreshPendingPairs();
		});
		var unsubResolved = onEvent("node.pair.resolved", () => {
			refreshAll();
		});
		var unsubDevice = onEvent("device.pair.resolved", () => {
			refreshAll();
		});

		return () => {
			unsub();
			unsubPair();
			unsubResolved();
			unsubDevice();
		};
	}, []);

	return html`<div class="flex-1 flex flex-col min-w-0 p-4 gap-3 overflow-y-auto">
		<div class="flex items-center justify-between">
			<h2 class="text-lg font-medium text-[var(--text-strong)]">Nodes</h2>
			<button
				class="provider-btn-secondary text-xs px-2 py-1"
				onClick=${refreshAll}
				disabled=${loading.value}
			>
				${loading.value ? "Refreshing..." : "Refresh"}
			</button>
		</div>

		<${TabBar} />

		${activeTab.value === "connected" ? html`<${ConnectedNodesList} />` : null}
		${activeTab.value === "paired" ? html`<${PairedDevicesList} />` : null}
		${activeTab.value === "pending" ? html`<${PendingPairsList} />` : null}

		<${Toasts} />
		<${ConfirmDialog} />
	</div>`;
}

// ── Mount / unmount ─────────────────────────────────────────

var _mounted = false;
var containerRef = null;

export function initNodes(container) {
	_mounted = true;
	containerRef = container;
	render(html`<${NodesPage} />`, container);
}

export function teardownNodes() {
	_mounted = false;
	if (containerRef) render(null, containerRef);
	containerRef = null;
}
