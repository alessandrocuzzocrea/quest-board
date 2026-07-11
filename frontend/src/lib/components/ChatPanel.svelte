<script lang="ts">
	import { api } from '$lib/api';

	let {
		boardId = '',
		cardId = undefined as string | undefined,
		open = false,
		onclose = undefined as (() => void) | undefined,
	}: { boardId: string; cardId?: string; open: boolean; onclose?: () => void } = $props();

	interface Message {
		role: 'user' | 'assistant';
		content: string;
	}

	let messages = $state<Message[]>([]);
	let inputText = $state('');
	let loading = $state(false);
	let loaded = $state(false);

	async function loadHistory() {
		if (!open || loaded) return;
		loaded = true;
		try {
			const params = new URLSearchParams({ board_id: boardId });
			if (cardId) params.set('card_id', cardId);
			const res = await api<{ messages: Message[] }>(`/ai/chat/history?${params}`);
			messages = res.messages ?? [];
		} catch {
			messages = [];
		}
	}

	async function send() {
		const text = inputText.trim();
		if (!text || loading) return;
		inputText = '';
		messages = [...messages, { role: 'user', content: text }];
		loading = true;
		try {
			const res = await api<{ reply: string }>('/ai/chat', {
				method: 'POST',
				body: JSON.stringify({ messages, board_id: boardId, card_id: cardId ?? null }),
			});
			messages = [...messages, { role: 'assistant', content: res.reply }];
		} catch {
			messages = [...messages, { role: 'assistant', content: 'Sorry, something went wrong. Please try again.' }];
		} finally {
			loading = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onclose?.();
		}
	}

	function close() {
		onclose?.();
	}

	function handleInputKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			send();
		}
	}

	$effect(() => {
		if (open) loadHistory();
		if (!open) loaded = false;
	});
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<div class="overlay" onclick={close} role="presentation"></div>
	<div class="panel" class:open role="dialog" aria-label="AI Chat">
		<header class="panel-header">
			<h2 class="panel-title">AI Chat</h2>
			<button class="close-btn" onclick={close} aria-label="Close">&times;</button>
		</header>

		<div class="panel-body">
			{#if messages.length === 0 && !loading}
				<div class="empty-state">Ask a question about the board or card.</div>
			{/if}

			<div class="messages" class:has-messages={messages.length > 0}>
				{#each messages as msg (msg)}
					<div class="message {msg.role}" data-role={msg.role}>
						<div class="bubble">{msg.content}</div>
					</div>
				{/each}

				{#if loading}
					<div class="message assistant">
						<div class="bubble loading-indicator">
							<span class="dot"></span>
							<span class="dot"></span>
							<span class="dot"></span>
						</div>
					</div>
				{/if}
			</div>
		</div>

		<div class="input-area">
			<textarea
				class="chat-input"
				bind:value={inputText}
				onkeydown={handleInputKeydown}
				placeholder="Type a message..."
				rows="1"
				disabled={loading}
			></textarea>
			<button class="send-btn" onclick={send} disabled={!inputText.trim() || loading}>
				Send
			</button>
		</div>
	</div>
{/if}

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: rgba(0,0,0,0.3);
		z-index: 100;
	}
	.panel {
		position: fixed;
		top: 0;
		right: 0;
		height: 100vh;
		width: 420px;
		max-width: 100vw;
		background: white;
		z-index: 101;
		display: flex;
		flex-direction: column;
		box-shadow: -4px 0 20px rgba(0,0,0,0.15);
		transform: translateX(100%);
		transition: transform 0.2s ease;
	}
	.panel.open {
		transform: translateX(0);
	}
	.panel-header {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 16px 20px;
		border-bottom: 1px solid #eee;
		flex-shrink: 0;
	}
	.panel-title {
		flex: 1;
		font-size: 18px;
		font-weight: 700;
		margin: 0;
	}
	.close-btn {
		background: none;
		border: none;
		font-size: 20px;
		cursor: pointer;
		color: #888;
		padding: 4px 8px;
		border-radius: 4px;
		flex-shrink: 0;
	}
	.close-btn:hover {
		background: #f0f0f0;
	}
	.panel-body {
		flex: 1;
		overflow-y: auto;
		padding: 16px 20px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.empty-state {
		color: #999;
		font-size: 13px;
		font-style: italic;
		text-align: center;
		margin-top: 40px;
	}
	.messages {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.messages.has-messages {
		flex: 1;
		justify-content: flex-end;
	}
	.message {
		display: flex;
	}
	.message.user {
		justify-content: flex-end;
	}
	.message.assistant {
		justify-content: flex-start;
	}
	.bubble {
		max-width: 75%;
		padding: 10px 14px;
		border-radius: 12px;
		font-size: 14px;
		line-height: 1.4;
		word-wrap: break-word;
		white-space: pre-wrap;
	}
	.message.user .bubble {
		background: #0079bf;
		color: white;
		border-bottom-right-radius: 4px;
	}
	.message.assistant .bubble {
		background: #f0f0f0;
		color: #333;
		border-bottom-left-radius: 4px;
	}
	.loading-indicator {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 12px 18px;
	}
	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: #999;
		animation: pulse 1.4s infinite ease-in-out;
	}
	.dot:nth-child(2) {
		animation-delay: 0.2s;
	}
	.dot:nth-child(3) {
		animation-delay: 0.4s;
	}
	@keyframes pulse {
		0%, 80%, 100% { opacity: 0.3; transform: scale(0.8); }
		40% { opacity: 1; transform: scale(1); }
	}
	.input-area {
		display: flex;
		gap: 8px;
		padding: 12px 20px;
		border-top: 1px solid #eee;
		flex-shrink: 0;
	}
	.chat-input {
		flex: 1;
		padding: 8px 12px;
		border: 1px solid #ddd;
		border-radius: 8px;
		font-size: 14px;
		font-family: inherit;
		resize: none;
		outline: none;
		line-height: 1.4;
		box-sizing: border-box;
	}
	.chat-input:focus {
		border-color: #0079bf;
	}
	.send-btn {
		padding: 8px 16px;
		background: #0079bf;
		color: white;
		border: none;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 600;
		cursor: pointer;
		flex-shrink: 0;
	}
	.send-btn:hover:not(:disabled) {
		background: #005f99;
	}
	.send-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
