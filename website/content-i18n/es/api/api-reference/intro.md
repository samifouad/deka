---
title: Descripción general de la API
description: Referencia de API REST para Tana Services
sidebar:
  order: 1
---
Este documento de referencia documenta las API REST expuestas por los servicios de Tana. Como **soberano** que ejecuta su propia red, utilizará estas API para integrarse con su infraestructura, construir paneles de control y crear herramientas personalizadas.

Tana proporciona dos API REST principales. Todos los puntos finales devuelven respuestas en JSON.

## URLs Base

### Servicio de Ledger (API de Blockchain)

```
http://localhost:8080
```

Maneja todas las operaciones de blockchain: usuarios, saldos, transacciones, bloques y contratos inteligentes.

### Servicio de Identidad (API de Autenticación)

```
http://localhost:8090
```

Maneja la autenticación mediante código QR móvil y la gestión de sesiones. Las claves privadas permanecen exclusivamente en dispositivos móviles por razones de seguridad.

Para implementaciones en producción, reemplace con las URLs de su servidor (por ejemplo, `https://api.tana.network` y `https://auth.tana.network`).

## Autenticación y Firmas

Todas las operaciones que cambian el estado (POST, PATCH, DELETE) requieren firmas criptográficas para probar la propiedad.

### Firmas de Transacción

**Campos Requeridos:**

```json
{
  "type": "transfer",
  "from": "usr_abc123",
  "to": "usr_xyz789",
  "timestamp": 1699459200,
  "nonce": 42,
  "amount": "100",
  "currencyCode": "USD",
  "signature": "ed25519_sig_..."
}
```

**Cómo Funciona la Firma:**

```typescript
import {
  signMessage,
  createTransactionMessage
} from '@tananetwork/crypto'

// 1. Create canonical transaction message
const tx = {
  type: 'transfer',
  from: 'usr_abc123',
  to: 'usr_xyz789',
  timestamp: Date.now(),
  nonce: 42,
  amount: '100',
  currencyCode: 'USD'
}

// 2. Sign with private key (SHA-256 + Ed25519)
const message = createTransactionMessage(tx)
const signature = await signMessage(message, privateKey)

// 3. Send to API with signature
POST /transactions {
  ...tx,
  signature
}
```

**Verificación del Servidor:**

La API verifica:
1. La firma es válida para el mensaje canónico
2. La firma coincide con la clave pública registrada para el usuario `from`
3. La marca de tiempo está dentro del rango aceptable (previene transacciones antiguas)
4. El nonce no se ha utilizado antes (previene ataques de repetición)

**Estado de Implementación:**
- ✅ Generación de par de claves Ed25519 - `@tananetwork/crypto`
- ✅ Firma de transacciones - CLI y móvil
- ✅ Verificación de firmas - Todos los servicios
- ✅ Protección contra repetición basada en nonce

## Formato de Respuesta

Todas las respuestas de la API siguen este formato:

### Respuesta de Éxito

```json
{
  "data": { ... },
  "status": "success"
}
```

### Respuesta de Error

```json
{
  "error": "Error message describing what went wrong",
  "timestamp": "2024-11-07T00:00:00.000Z"
}
```

## Códigos de Estado HTTP

| Código | Significado |
|--------|-------------|
| `200` | Éxito - Solicitud completada con éxito |
| `201` | Creado - Recurso creado con éxito |
| `400` | Solicitud Incorrecta - Parámetros de solicitud inválidos |
| `404` | No Encontrado - El recurso no existe |
| `500` | Error Interno del Servidor - Algo salió mal en el servidor |

## Inicio Rápido

### Verificación de Salud

```bash
curl http://localhost:8080/health
```

```json
{
  "status": "ok"
}
```

### Obtener Información del Servicio

```bash
curl http://localhost:8080/
```

```json
{
  "service": "tana-ledger",
  "version": "0.1.0",
  "status": "healthy",
  "timestamp": "2024-11-07T00:00:00.000Z"
}
```

## Categorías de API

### Identidad y Autenticación (Puerto 8090)
Autenticación mediante código QR móvil con seguridad respaldada por hardware.

- **POST /auth/session/create** - Crear sesión de inicio de sesión con código QR
- **GET /auth/session/:id/events** - Flujo SSE para actualizaciones en tiempo real
- **GET /auth/session/:id** - Obtener detalles de la sesión (móvil escanea QR)
- **POST /auth/session/:id/approve** - Aprobar inicio de sesión con firma Ed25519
- **POST /auth/session/:id/reject** - Rechazar intento de inicio de sesión
- **POST /auth/session/validate** - Validar token de sesión

**Modelo de Seguridad:** Claves privadas SOLO en dispositivos móviles. El escritorio/portátil recibe tokens de sesión únicamente.

[Ver API de Identidad →](/tana-api/identity/)

---

### Usuarios (Puerto 8080)
Gestionar cuentas de usuario en la blockchain.

- **POST /users** - Crear un nuevo usuario
- **GET /users** - Listar todos los usuarios
- **GET /users/:id** - Obtener usuario por ID
- **GET /users/by-username/:username** - Obtener usuario por nombre de usuario

[Ver API de Usuarios →](/tana-api/users/)

### Saldos (Puerto 8080)
Consultar y gestionar saldos en múltiples monedas.

- **GET /balances** - Consultar saldos
- **POST /balances** - Establecer saldo
- **GET /balances/currencies** - Listar monedas soportadas
- **POST /balances/currencies/seed** - Sembrar monedas predeterminadas

[Ver API de Saldos →](/tana-api/balances/)

### Transacciones (Puerto 8080)
Crear y consultar transacciones en la blockchain.

- **POST /transactions** - Crear transacción
- **GET /transactions** - Listar transacciones
- **GET /transactions/:id** - Obtener transacción por ID

[Ver API de Transacciones →](/tana-api/transactions/)

### Bloques (Puerto 8080)
Consultar bloques y el historial de la blockchain.

- **GET /blocks** - Listar bloques
- **GET /blocks/latest** - Obtener el bloque más reciente
- **GET /blocks/:height** - Obtener bloque por altura

[Ver API de Bloques →](/tana-api/blocks/)

### Contratos (Puerto 8080)
Desplegar y gestionar contratos inteligentes.

- **POST /contracts** - Desplegar contrato
- **GET /contracts** - Listar contratos
- **GET /contracts/:id** - Obtener contrato por ID

[Ver API de Contratos →](/tana-api/contracts/)

## Limitación de Tasa

Actualmente, no hay límites de tasa en la API de desarrollo. Las implementaciones en producción deben implementar limitación de tasa a nivel de proxy inverso (por ejemplo, Nginx).

## CORS

La API admite CORS para todos los orígenes en modo de desarrollo:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Credentials: true
```

## Paginación

Los puntos finales de lista admiten paginación a través de parámetros de consulta:

```bash
GET /users?page=1&limit=20
```

El límite predeterminado es de 100 elementos por página.

## Ejemplos

### Crear Usuario y Verificar Saldo

```bash
# 1. Create user
curl -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{
    "publicKey": "ed25519_abc123...",
    "username": "@alice",
    "displayName": "Alice Johnson"
  }'

# Response:
# {
#   "userId": "user_xyz789",
#   "transactionId": "tx_abc123",
#   "status": "pending"
# }

# 2. Get balance (after block inclusion)
curl "http://localhost:8080/balances?ownerId=user_xyz789&currencyCode=USD"

# Response:
# {
#   "ownerId": "user_xyz789",
#   "ownerType": "user",
#   "currencyCode": "USD",
#   "amount": "1000.00",
#   "updatedAt": "2024-11-07T00:00:00.000Z"
# }
```

## Próximos Pasos

- [API de Identidad](/tana-api/identity/) - Autenticación móvil mediante código QR
- [API de Usuarios](/tana-api/users/) - Gestión de cuentas de usuario
- [API de Saldos](/tana-api/balances/) - Consultas y actualizaciones de saldo
- [API de Transacciones](/tana-api/transactions/) - Creación y consultas de transacciones
- [API de Bloques](/tana-api/blocks/) - Consultas de blockchain
- [API de Contratos](/tana-api/contracts/) - Despliegue de contratos inteligentes
- [Referencia de CLI](/tana-cli/intro/) - Interfaz de línea de comandos
