# USB Manager - Forensic Device Tracker

**USB Manager** es una aplicación de escritorio de alto rendimiento construida sobre **Tauri v2**, diseñada para el monitoreo forense, auditoría y rastreo histórico de dispositivos de almacenamiento USB.

La aplicación permite detectar conexiones en tiempo real, registrar la actividad de los puertos, mantener un inventario de dispositivos (Device Vault) y generar "snapshots" (instantáneas) de la estructura de archivos de los dispositivos para su revisión posterior, incluso cuando el dispositivo ya ha sido desconectado.

---

## 🚀 Características Principales

### 1. Monitoreo en Tiempo Real (Live Feed)
- Detección instantánea de eventos `CONNECT` y `DISCONNECT`.
- Identificación de dispositivos mediante VID (Vendor ID), PID (Product ID) y Número de Serie.
- **Estrategia Híbrida de Detección:** Combina la lectura de bajo nivel (libusb) con el sistema de archivos del SO (sysinfo) para garantizar la detección en Windows, incluso cuando los controladores del sistema bloquean el acceso directo al hardware.

### 2. Device Vault (Bóveda de Dispositivos)
- Registro persistente de todo dispositivo que haya tocado el sistema.
- Historial de capacidad, fabricante y fechas de última conexión.
- Búsqueda y filtrado de dispositivos históricos.

### 3. Indexado Forense (File Snapshots)
- Al conectar un dispositivo, el sistema escanea automáticamente la estructura de archivos en un hilo secundario (sin congelar la UI).
- Guarda metadatos (Nombre, Ruta, Tamaño, Extensión, Tipo) en una base de datos local **SQLite**.
- **Privacidad:** No se copia el contenido de los archivos, solo la estructura y metadatos.

### 4. Cronología y Auditoría
- Visualización de sesiones de conexión en una línea de tiempo (Timeline).
- Exploración de archivos "Offline": Permite ver qué archivos contenía un USB en una fecha específica del pasado.

---

## 🛠️ Stack Tecnológico

La arquitectura sigue el modelo de Tauri: un backend ligero y seguro en Rust con un frontend web moderno.

### Backend (Core)
- **Lenguaje:** [Rust](https://www.rust-lang.org/)
- **Framework:** [Tauri 2.0](https://tauri.app/)
- **Base de Datos:** SQLite (vía `rusqlite`).
- **Librerías Clave:**
  - `rusb`: Acceso a dispositivos USB a bajo nivel.
  - `sysinfo`: Información de discos y puntos de montaje.
  - `walkdir`: Escaneo recursivo de directorios optimizado.
  - `tokio`: Runtime asíncrono para el manejo de eventos y tareas en segundo plano.

### Frontend (UI)
- **Framework:** [SvelteKit](https://kit.svelte.dev/) (Svelte 5).
- **Lenguaje:** TypeScript.
- **Estilos:** TailwindCSS.
- **UI Kit:** Componentes personalizados basados en conceptos de Shadcn.
- **Iconos:** Lucide-svelte.

---

## ⚙️ Instalación y Desarrollo

### Prerrequisitos
1.  **Node.js** (v18 o superior).
2.  **Rust & Cargo**: [Instalar Rust](https://rustup.rs/).
3.  **Dependencias de Compilación del Sistema**:
    - **Windows:** Microsoft Visual Studio C++ Build Tools.
    - **Linux:** `libwebkit2gtk-4.0-dev`, `build-essential`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`.
    - **macOS:** Xcode Command Line Tools.

### Pasos para ejecutar
1.  Clonar el repositorio:
    ```bash
    git clone <url-del-repo>
    cd usb-manager
    ```

2.  Instalar dependencias del frontend:
    ```bash
    npm install
    # o si usas bun
    bun install
    ```

3.  Ejecutar en modo desarrollo:
    ```bash
    npm run tauri dev
    # o
    bun run tauri dev
    ```
    *Esto compilará el backend de Rust y lanzará la ventana de la aplicación junto con el servidor de desarrollo de Vite.*

---

## 🏗️ Arquitectura y Funcionamiento Interno

### Lógica de Detección USB (`usb_monitor.rs`)
El mayor desafío en aplicaciones USB de escritorio es el bloqueo de controladores en Windows. Este proyecto utiliza una lógica de **"Fail-safe Detection"**:

1.  **Escaneo de Discos:** Primero consulta a `sysinfo` por discos removibles montados (ej. `E:\`). Esto es infalible para detectar almacenamiento.
2.  **Enriquecimiento de Datos:** Luego consulta a `rusb` para intentar obtener datos del fabricante (Serial, VID, PID).
3.  **Fusión de Datos:**
    - Si `rusb` logra leer el dispositivo, se usan los datos técnicos precisos.
    - Si Windows bloquea `rusb`, el sistema genera un ID único basado en el punto de montaje y el tamaño total del disco (`DISK_E_16GB`).
    - **Resultado:** El dispositivo siempre aparece en el dashboard y se registra en la base de datos, garantizando la auditoría.

### Base de Datos (`db.rs`)
La base de datos se inicializa automáticamente en el directorio `AppLocalData` del usuario.

| Tabla | Descripción |
| :--- | :--- |
| `devices` | Catálogo único de dispositivos (Serial, Fabricante, Capacidad). |
| `activity_log` | Registro de cada evento de conexión/desconexión con timestamp. |
| `file_snapshots` | Índice masivo de archivos vinculados a una entrada de `activity_log`. |

---

## 📂 Estructura del Proyecto

```plaintext
usb-manager/
├── src/                        # Frontend (SvelteKit)
│   ├── lib/
│   │   ├── components/         # UI Components (Cards, Tables, Feed)
│   │   └── utils/              # Helpers
│   └── routes/                 # Páginas (Dashboard, Detalle Dispositivo)
│       ├── +page.svelte        # Dashboard Principal
│       └── devices/            # Bóveda de Dispositivos
│
├── src-tauri/                  # Backend (Rust)
│   ├── src/
│   │   ├── db.rs               # Manejo de SQLite
│   │   ├── file_scanner.rs     # Escaneo recursivo de archivos
│   │   ├── lib.rs              # Exportación de comandos a JS
│   │   ├── main.rs             # Entry point
│   │   └── usb_monitor.rs      # Lógica de detección hardware
│   ├── capabilities/           # Permisos de seguridad Tauri
│   └── tauri.conf.json         # Configuración de la ventana y app
│
└── package.json