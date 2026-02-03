<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  interface UsbDevice {
    id: number;
    vendor_id: number;
    product_id: number;
    product_name?: string;
    manufacturer_name?: string;
    serial_number?: string;
  }

  let name = $state("");
  let greetMsg = $state("");
  let connectedDevices = $state<UsbDevice[]>([]);
  let eventLog = $state<string[]>([]);

  function addEvent(message: string) {
    const timestamp = new Date().toLocaleTimeString();
    eventLog = [`[${timestamp}] ${message}`, ...eventLog];
    if (eventLog.length > 20) {
      eventLog = eventLog.slice(0, 20);
    }
  }

  async function greet(event: Event) {
    event.preventDefault();
    greetMsg = await invoke("greet", { name });
  }

  async function loadConnectedDevices() {
    try {
      const devices = await invoke<UsbDevice[]>("get_connected_devices");
      connectedDevices = devices;
      addEvent(`Loaded ${devices.length} connected devices`);
    } catch (error) {
      console.error("Failed to load connected devices:", error);
    }
  }

  function formatDevice(device: UsbDevice): string {
    const name = device.product_name || device.manufacturer_name || `Device ${device.id}`;
    return `${name} (VID: ${device.vendor_id.toString(16).padStart(4, '0').toUpperCase()}, PID: ${device.product_id.toString(16).padStart(4, '0').toUpperCase()})`;
  }

  onMount(async () => {
    await loadConnectedDevices();
    
    // Listen for USB connect events
    await listen<UsbDevice>("usb-connected", (event) => {
      const device = event.payload;
      addEvent(`USB Connected: ${formatDevice(device)}`);
      loadConnectedDevices();
    });
    
    // Listen for USB disconnect events
    await listen<UsbDevice>("usb-disconnected", (event) => {
      const device = event.payload;
      addEvent(`USB Disconnected: ${formatDevice(device)}`);
      loadConnectedDevices();
    });
  });
</script>

<main class="container">
  <h1>USB Device Manager</h1>

  <div class="row">
    <a href="https://vite.dev" target="_blank">
      <img src="/vite.svg" class="logo vite" alt="Vite Logo" />
    </a>
    <a href="https://tauri.app" target="_blank">
      <img src="/tauri.svg" class="logo tauri" alt="Tauri Logo" />
    </a>
    <a href="https://svelte.dev" target="_blank">
      <img src="/svelte.svg" class="logo svelte-kit" alt="SvelteKit Logo" />
    </a>
  </div>

  <div class="usb-section">
    <h2>Connected USB Devices</h2>
    <div class="device-list">
      {#if connectedDevices.length > 0}
        {#each connectedDevices as device}
          <div class="device-card">
            <div class="device-name">
              {device.product_name || device.manufacturer_name || `Unknown Device`}
            </div>
            <div class="device-details">
              <strong>ID:</strong> {device.id}<br>
              <strong>Vendor ID:</strong> {device.vendor_id.toString(16).padStart(4, '0').toUpperCase()}<br>
              <strong>Product ID:</strong> {device.product_id.toString(16).padStart(4, '0').toUpperCase()}<br>
              {#if device.serial_number}
                <strong>Serial:</strong> {device.serial_number}<br>
              {/if}
            </div>
          </div>
        {/each}
      {:else}
        <p>No USB devices connected</p>
      {/if}
    </div>
  </div>

  <div class="usb-section">
    <h2>Event Log</h2>
    <button onclick={() => eventLog = []}>Clear Log</button>
    <div class="event-log">
      {#if eventLog.length > 0}
        {#each eventLog as event}
          <div class="event-item">{event}</div>
        {/each}
      {:else}
        <p>No USB events yet. Connect or disconnect a USB device to see events.</p>
      {/if}
    </div>
  </div>

  <div class="test-section">
    <h2>Test Greeting</h2>
    <form class="row" onsubmit={greet}>
      <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
      <button type="submit">Greet</button>
    </form>
    <p>{greetMsg}</p>
  </div>
</main>

<style>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.svelte-kit:hover {
  filter: drop-shadow(0 0 2em #ff3e00);
}

.usb-section {
  margin: 20px 0;
  padding: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background-color: #f9f9f9;
}

.device-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 15px;
  margin-top: 15px;
}

.device-card {
  padding: 15px;
  border: 1px solid #ccc;
  border-radius: 8px;
  background-color: white;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.device-name {
  font-weight: bold;
  font-size: 1.1em;
  margin-bottom: 8px;
  color: #333;
}

.device-details {
  font-size: 0.9em;
  color: #666;
  line-height: 1.4;
}

.event-log {
  margin-top: 15px;
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid #ccc;
  background-color: white;
}

.event-item {
  padding: 8px 12px;
  border-bottom: 1px solid #eee;
  font-family: monospace;
  font-size: 0.9em;
}

.event-item:last-child {
  border-bottom: none;
}

.test-section {
  margin-top: 30px;
  padding: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background-color: #f0f8ff;
}

:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>
