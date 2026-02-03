import os
import re
import json

# ------------------------------------------------------------------
# CONFIGURATION
# ------------------------------------------------------------------
ROOT_DIR = os.path.dirname(os.path.abspath(__file__))
BASE_OUTPUT_FILE = "consolidated_code"
OUTPUT_EXTENSION = ".txt"

# Extensiones permitidas
ALLOWED_EXTENSIONS = (".rs", ".toml", ".json", ".mjs", ".js", ".md", ".ts", ".svelte", '.html')

# Tamaño máximo por archivo (en bytes)
# 500 KB = 500 * 1024
MAX_SIZE_PER_FILE = 500 * 1024 

# Elementos a excluir
EXCLUDED = {
    "node_modules", ".git", ".next", "dist", "build", ".cache", "coverage",
    ".vscode", ".idea", "consolidated_code", "consolidate.py", "consolidate.js",
    ".env", ".env.local", ".env.production", ".env.development",
    "package-lock.json", "yarn.lock", "pnpm-lock.yaml", "pnpm-workspace.yaml",
    "public", ".dockerignore", ".gitignore", "TODO.md", "__pycache__", "target"
}

# ------------------------------------------------------------------

def is_source_file(filename):
    return filename.endswith(ALLOWED_EXTENSIONS)

def should_exclude(full_path):
    """Excluye si alguna parte de la ruta está en EXCLUDED"""
    return any(part in EXCLUDED for part in full_path.replace(ROOT_DIR, "").split(os.sep) if part)

def get_next_output_path(part_number):
    if part_number == 1:
        return f"{BASE_OUTPUT_FILE}{OUTPUT_EXTENSION}"
    else:
        return f"{BASE_OUTPUT_FILE}_part{part_number}{OUTPUT_EXTENSION}"

def minify_content(file_path, content):
    """
    Minifica el contenido según la extensión del archivo.
    """
    ext = os.path.splitext(file_path)[1].lower()

    # 1. Minificación para JSON
    if ext == '.json':
        try:
            # Carga el JSON y lo vuelca sin espacios (separators reducidos)
            obj = json.loads(content)
            return json.dumps(obj, separators=(',', ':'))
        except:
            return content.strip()

    # 2. Minificación para JS/TS
    # Eliminar comentarios de bloque /* ... */
    content = re.sub(r'/\*[\s\S]*?\*/', '', content)
    
    # Procesar línea por línea
    minified_lines = []
    for line in content.splitlines():
        # Quitar espacios al inicio y final
        line = line.strip()
        
        # Ignorar líneas vacías
        if not line:
            continue
            
        # Ignorar líneas que son puramente comentarios (//)
        if line.startswith('//'):
            continue
            
        minified_lines.append(line)
    
    return '\n'.join(minified_lines)

def process_directory(directory, process_file_callback):
    """Recorre el directorio"""
    try:
        with os.scandir(directory) as entries:
            for entry in entries:
                if should_exclude(entry.path):
                    continue

                if entry.is_dir():
                    process_directory(entry.path, process_file_callback)
                elif entry.is_file() and is_source_file(entry.name):
                    process_file_callback(entry.path)
    except OSError as e:
        print(f"Error accessing directory {directory}: {e}")

def consolidate_code():
    print(f"Starting MINIFIED consolidation from: {ROOT_DIR}")
    print(f"Max size per file: {MAX_SIZE_PER_FILE / 1024:.1f} KB")

    current_part = 1
    current_size = 0
    current_output_path = get_next_output_path(current_part)
    out_file = None

    def open_new_file():
        nonlocal out_file, current_size, current_part, current_output_path
        if out_file:
            out_file.close()
        
        current_output_path = get_next_output_path(current_part)
        out_file = open(current_output_path, "w", encoding="utf-8")
        current_size = 0
        print(f"Creating new part: {current_output_path}")

    def process_file_callback(file_path):
        nonlocal current_size, current_part

        rel_path = os.path.relpath(file_path, ROOT_DIR)
        
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                raw_content = f.read()
            
            # --- FASE DE MINIFICACIÓN ---
            minified_content = minify_content(file_path, raw_content)
            
            # Si tras minificar está vacío, lo saltamos
            if not minified_content:
                return

            # Cabecera mínima: // FILE: ruta/archivo
            header = f"\n// FILE: {rel_path}\n"
            
            full_block = header + minified_content
            block_size = len(full_block.encode('utf-8')) # Tamaño real en bytes

            # Si el archivo supera el límite, crear nuevo archivo
            if current_size + block_size > MAX_SIZE_PER_FILE and current_size > 0:
                current_part += 1
                open_new_file()

            out_file.write(full_block)
            current_size += block_size

        except Exception as err:
            print(f"Error reading/processing file {file_path}: {err}")

    # Iniciar el primer archivo
    open_new_file()

    try:
        process_directory(ROOT_DIR, process_file_callback)
        
        if out_file:
            out_file.close()

        # Resumen final
        print("-" * 30)
        if current_part == 1:
            if os.path.exists(current_output_path):
                final_size = os.path.getsize(current_output_path)
                print(f"Consolidation complete!")
                print(f"Single MINIFIED file: {current_output_path}")
                print(f"Final size: {final_size / 1024:.2f} KB ({final_size:,} bytes)")
            else:
                print("No content found.")
        else:
            print(f"Consolidation complete! Split into {current_part} files:")
            total_size = 0
            for i in range(1, current_part + 1):
                path = get_next_output_path(i)
                if os.path.exists(path):
                    size = os.path.getsize(path)
                    total_size += size
                    print(f"  {path} → {size / 1024:.2f} KB ({size:,} bytes)")
            print(f"Total consolidated size: {total_size / 1024:.2f} KB ({total_size:,} bytes)")

    except Exception as e:
        print(f"Error during consolidation: {e}")
        if out_file:
            out_file.close()

# ------------------------------------------------------------------
# RUN
# ------------------------------------------------------------------
if __name__ == "__main__":
    consolidate_code()