# Plan de Implementación de IPC (Comunicación entre Procesos)

## Visión General

Para hacer de deka un verdadero reemplazo directo de Node.js, necesitamos implementar IPC bidireccional entre procesos padre e hijo. Esto está bloqueando actualmente a Next.js para ejecutarse de manera nativa en el runtime de deka.

## ⚠️ Aclaración de Terminología

**Cuando este documento dice "stdio"**, se refiere al **concepto de flujos estándar a nivel de OS** (descriptores de archivo):
- `stdin` = descriptor de archivo 0 (entrada estándar)
- `stdout` = descriptor de archivo 1 (salida estándar)
- `stderr` = descriptor de archivo 2 (error estándar)
- `fd 3` = descriptor de archivo 3 (utilizado para el canal IPC)

**Esto NO es la `deka-stdio` crate** (que es para registrar/formatear salida).

Para evitar confusiones, este documento usará:
- **"descriptores de archivo"** o **"fd 0/1/2/3"** al referirse a flujos de OS
- **"flujos estándar"** al referirse a stdin/stdout/stderr
- **"`deka-stdio` crate"** al referirse a la biblioteca de registro

## Estado Actual

### Lo que Funciona ✓
- ✓ Creación de procesos de manera sincrónica (`op_process_spawn_immediate`)
- ✓ PID disponible inmediatamente después del fork
- ✓ Soporte `stdio: 'inherit'` (heredar stdin/stdout/stderr del padre)
- ✓ Paso de variables de entorno
- ✓ Delegación de fork a Node.js (solución temporal)

### Lo que Falta ✗
- ✗ Configuración del canal IPC (descriptor de archivo 3 / fd 3)
- ✗ `child.send(message)` - Mensajería de padre a hijo
- ✗ `child.on('message', callback)` - Mensajería de hijo a padre
- ✗ Serialización/deserialización de mensajes
- ✗ Algoritmo de clonación estructurada para el paso de mensajes
- ✗ Transferencia de manejadores (para pasar sockets, servidores, etc.)

## Cómo Funciona el IPC en Node.js

### Canal IPC (Descriptor de Archivo 3)
Cuando Node.js crea un hijo con `child_process.fork()`, este:
1. Crea un canal de comunicación adicional en **descriptor de archivo 3** (fd 3)
2. Descriptores de archivo estándar:
   - fd 0 = stdin (entrada estándar)
   - fd 1 = stdout (salida estándar)
   - fd 2 = stderr (error estándar)
   - **fd 3 = canal IPC** (tubería de mensajes bidireccional)
3. Utiliza este canal para el paso de mensajes JSON bidireccional
4. Tanto el padre como el hijo pueden enviar/recibir en este canal

### Formato del Mensaje
Los mensajes están serializados en JSON con un protocolo específico:
```javascript
{
  cmd: 'NODE_...',  // Internal command type (optional)
  // ... message data ...
}
```

Los mensajes de usuario se envían como:
```javascript
{
  cmd: 'NODE_HANDLE',  // or no cmd field for simple messages
  msg: <actual user data>
}
```

### Superficie de la API

**Proceso Padre:**
```javascript
const child = fork('./worker.js');

// Send message to child
child.send({ hello: 'world' });

// Receive message from child
child.on('message', (msg) => {
  console.log('Parent received:', msg);
});
```

**Proceso Hijo:**
```javascript
// Send message to parent
process.send({ hello: 'parent' });

// Receive message from parent
process.on('message', (msg) => {
  console.log('Child received:', msg);
});
```

## Diseño de Arquitectura

### Lado de Rust (process.rs)

#### Nuevas Estructuras
```rust
// IPC channel handle
struct IpcChannel {
    reader: AsyncMutex<ChildStdout>,  // stdio[3] for reading
    writer: AsyncMutex<ChildStdin>,   // stdio[3] for writing
}

// Updated ChildProcessEntry
struct ChildProcessEntry {
    child: Option<Child>,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
    stdin: Option<ChildStdin>,
    ipc: Option<IpcChannel>,  // NEW
}
```

#### Nuevas Operaciones

**1. op_process_spawn_immediate con IPC**
```rust
#[op2]
#[bigint]
pub(super) fn op_process_spawn_immediate(
    // ... existing params ...
    #[serde] enable_ipc: bool,  // NEW
) -> Result<u64, CoreError> {
    let mut cmd = Command::new(command);

    if enable_ipc {
        // Set up file descriptors: [fd0, fd1, fd2, fd3]
        // fd 0 = stdin, fd 1 = stdout, fd 2 = stderr, fd 3 = IPC channel
        use std::process::Stdio;
        use std::os::unix::process::CommandExt;

        cmd.stdin(Stdio::piped());   // fd 0
        cmd.stdout(Stdio::piped());  // fd 1
        cmd.stderr(Stdio::piped());  // fd 2
        // TODO: Add 4th pipe for IPC channel at fd 3
        // This requires platform-specific code to set up an extra pipe
    }

    // ... spawn and store with IPC channel ...
}
```

**2. op_process_send_message**
```rust
#[op2(async)]
pub(super) async fn op_process_send_message(
    #[bigint] id: u64,
    #[string] message: String,  // JSON-serialized message
) -> Result<(), CoreError> {
    // Get the IPC channel for this process
    // Write message + newline delimiter
    // Flush the writer
}
```

**3. op_process_read_message**
```rust
#[op2(async)]
#[string]
pub(super) async fn op_process_read_message(
    #[bigint] id: u64,
) -> Result<Option<String>, CoreError> {
    // Read from IPC channel until newline
    // Return the JSON message string
    // Return None on EOF
}
```

### Lado de JavaScript (deka.js)

#### Actualizaciones de ChildProcess

```javascript
class ChildProcess extends EventEmitter {
    pid;
    stdout;
    stderr;
    stdin;
    stdio;
    channel;      // NEW: IPC channel (if enabled)
    connected;    // NEW: IPC connection state

    constructor(pidPromise, options) {
        super();
        this.connected = options?.ipc || false;
        // ...
    }

    send(message, sendHandle, options, callback) {
        if (!this.connected) {
            throw new Error('channel closed');
        }

        // Normalize arguments (Node.js supports multiple signatures)
        if (typeof sendHandle === 'function') {
            callback = sendHandle;
            sendHandle = undefined;
            options = undefined;
        }

        // Serialize message
        const serialized = JSON.stringify({
            cmd: 'NODE_HANDLE',
            msg: message
        });

        // Send via IPC channel
        op_process_send_message(this.pid, serialized)
            .then(() => {
                if (callback) callback(null);
            })
            .catch((err) => {
                if (callback) callback(err);
                else this.emit('error', err);
            });

        return true;
    }

    disconnect() {
        if (!this.connected) return;
        this.connected = false;
        this.emit('disconnect');
        // Close IPC channel
    }

    attachIpcReader() {
        if (!this.connected || !this.pid) return;

        const readLoop = async () => {
            while (this.connected) {
                try {
                    const msg = await op_process_read_message(this.pid);
                    if (msg === null) {
                        // EOF - channel closed
                        this.disconnect();
                        break;
                    }

                    // Deserialize and emit
                    const parsed = JSON.parse(msg);
                    const userMsg = parsed.msg !== undefined ? parsed.msg : parsed;
                    this.emit('message', userMsg);
                } catch (err) {
                    this.emit('error', err);
                    break;
                }
            }
        };

        readLoop();
    }
}
```

#### Actualizaciones de fork()

```javascript
function fork(modulePath, args = [], options) {
    // ... existing setup ...

    // Enable IPC by default for fork (like Node.js)
    const ipcOptions = {
        ...options,
        ipc: true,
        stdio: options?.stdio || ['pipe', 'pipe', 'pipe', 'ipc']
    };

    const child = new ChildProcess(undefined, ipcOptions);

    // ... spawn with IPC enabled ...

    return child;
}
```

#### process.send() para Proceso Hijo

```javascript
// In the child process (running in isolate)
globalThis.process.send = function(message, sendHandle, options, callback) {
    if (!globalThis.process.connected) {
        throw new Error('channel closed');
    }

    // Same implementation as ChildProcess.send
    // but uses a special "to parent" IPC channel
};

globalThis.process.connected = true;  // If forked with IPC
globalThis.process.channel = { ... };  // IPC channel reference

globalThis.process.disconnect = function() {
    globalThis.process.connected = false;
    globalThis.process.emit('disconnect');
};
```

## Fases de Implementación

### Fase 1: Configuración Básica del Canal IPC ✓ (Comienza Aquí)
**Objetivo:** Hacer funcionar la tubería stdio[3] para mensajería unidireccional

**Tareas:**
1. ✓ Investigar cómo agregar una 4ª tubería stdio en Rust tokio::process::Command
2. ✓ Actualizar `op_process_spawn_immediate` para crear stdio[3] cuando `enable_ipc: true`
3. ✓ Almacenar lector/escritor IPC en ChildProcessEntry
4. ✓ Implementar `op_process_send_message` (padre → hijo)
5. ✓ Implementar `op_process_read_message` (padre ← hijo)
6. ✓ Probar mensajería unidireccional con un script de trabajo simple

**Criterios de Éxito:**
- El padre puede enviar un mensaje JSON al hijo
- El hijo puede leer el mensaje desde stdin/fd 3
- Sin bloqueos, manejo de errores limpio

### Fase 2: Mensajería Bidireccional
**Objetivo:** Habilitar comunicación completa en dos vías

**Tareas:**
1. ✓ Implementar detección del canal IPC del lado del hijo (verificar si fd 3 existe)
2. ✓ Agregar `process.send()` al ámbito global del proceso hijo
3. ✓ Agregar manejador `process.on('message')` en el hijo
4. ✓ Implementar bucle de lectura de mensajes en el padre (ChildProcess.attachIpcReader)
5. ✓ Probar mensajería de ida y vuelta (padre ↔ hijo)

**Criterios de Éxito:**
- El padre envía → El hijo recibe y responde → El padre recibe
- Los emisores de eventos funcionan correctamente
- El orden de los mensajes se conserva

### Fase 3: API ChildProcess.send()
**Objetivo:** API completamente compatible con Node.js

**Tareas:**
1. ✓ Implementar `child.send(message, callback)`
2. ✓ Manejar múltiples firmas de callback (compatibilidad con Node.js)
3. ✓ Implementar `child.disconnect()` / `child.connected`
4. ✓ Emitir eventos 'disconnect'
5. ✓ Manejo de errores y casos límite

**Criterios de Éxito:**
- Todas las firmas de send() funcionan
- Disconnect cierra correctamente los canales
- Errores propagados correctamente

### Fase 4: Serialización de Mensajes y Características Avanzadas
**Objetivo:** Manejar tipos de datos complejos

**Tareas:**
1. ⬜ Implementar algoritmo de clonación estructurada (o usar el integrado de V8)
2. ⬜ Soportar referencias circulares
3. ⬜ Manejar tipos especiales (Buffer, arreglos tipados, objetos Error)
4. ⬜ Implementar transferencia de manejadores (opcional - avanzado)

**Criterios de Éxito:**
- Se pueden enviar objetos complejos
- Buffers serializados correctamente
- Los manejadores coinciden con el comportamiento de Node.js

### Fase 5: Integración de Next.js
**Objetivo:** Eliminar la solución alternativa de delegación de Node.js

**Tareas:**
1. ⬜ Eliminar la detección de Next.js de `run.rs`
2. ⬜ Actualizar fork() para habilitar IPC por defecto
3. ⬜ Probar el servidor de desarrollo de Next.js con el runtime nativo de deka
4. ⬜ Verificar que todos los mensajes IPC de Next.js funcionen (trabajador listo, servidor listo, etc.)

**Criterios de Éxito:**
- `deka run --deka dev` inicia Next.js de manera nativa
- Sin delegación a Node.js
- Paridad completa de características con `next dev`

## Consideraciones Específicas de la Plataforma

### Unix/Linux/macOS
- Usar `std::os::unix::process::CommandExt` para configurar fd 3
- Las tuberías funcionan de manera nativa

### Windows
- Windows no utiliza descriptores de archivo de la misma manera
- Puede necesitar `std::os::windows::process::CommandExt`
- Considerar tuberías nombradas u otros mecanismos IPC
- Diferir el soporte de Windows a una fase posterior si es necesario

## Casos de Prueba

### Pruebas Unitarias
```javascript
// test/ipc/basic-send.js
const { fork } = require('child_process');
const child = fork('./worker.js');

child.on('message', (msg) => {
  console.assert(msg.result === 42);
  child.kill();
});

child.send({ compute: 'meaning of life' });
```

```javascript
// test/ipc/worker.js
process.on('message', (msg) => {
  if (msg.compute) {
    process.send({ result: 42 });
  }
});
```

### Pruebas de Integración
- Probar con el servidor de desarrollo real de Next.js
- Probar con un administrador de procesos estilo PM2
- Probar el orden de los mensajes bajo carga
- Probar escenarios de desconexión/reconexión

## Métricas de Éxito

**Fase 1 Completa:** IPC unidireccional funcionando (1-2 días)
**Fase 2 Completa:** IPC bidireccional funcionando (1-2 días)
**Fase 3 Completa:** Paridad completa de API (1 día)
**Fase 4 Completa:** Características avanzadas (2-3 días)
**Fase 5 Completa:** Next.js funcionando de manera nativa (1 día de pruebas)

**Estimación Total:** 6-10 días de desarrollo enfocado

## Preguntas Abiertas

1. **Clonación Estructurada:** ¿Usar el integrado de V8 o implementar el nuestro?
2. **Transferencia de Manejadores:** ¿Necesitamos esto para Next.js? (Probablemente no inicialmente)
3. **Soporte para Windows:** ¿Diferir o implementar junto con Unix?
4. **Rendimiento:** ¿Deben los mensajes ser almacenados en búfer? ¿Cuál es el tamaño óptimo del búfer?
5. **ID del Proceso Hijo:** ¿Cómo detectar si se está ejecutando como hijo bifurcado? (¿Verificar existencia de fd 3?)

## Referencias

- [Documentación de Node.js child_process.fork()](https://nodejs.org/api/child_process.html#child_processforkmodulepath-args-options)
- [Implementación del canal IPC de Node.js (C++)](https://github.com/nodejs/node/blob/main/src/process_wrap.cc)
- [Protocolo de mensaje IPC](https://github.com/nodejs/node/blob/main/lib/internal/child_process.js)
- [Algoritmo de Clonación Estructurada](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm)

## Próximos Pasos

1. ✅ Comprometer el trabajo actual con este plan
2. **Comenzar la Fase 1:** Investigar e implementar la tubería stdio[3] en Rust
3. Crear un caso de prueba mínimo (el padre envía un mensaje, el hijo lo registra)
4. Iterar sobre las fases de implementación
5. Eliminar la delegación de Next.js una vez que IPC esté funcionando

---

**Estado:** Planificación Completa - Listo para Comenzar la Implementación
**Prioridad:** Alta (bloquea el soporte nativo de Next.js)
**Complejidad:** Media-Alta (nuevo subsistema pero bien definido)
