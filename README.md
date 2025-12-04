# 🛡️ LaMuralla: Cognitive GraphRAG Engine

![Rust](https://img.shields.io/badge/Backend-Rust_1.81+-orange?style=for-the-badge&logo=rust)
![Neo4j](https://img.shields.io/badge/Graph_DB-Neo4j_5+-008CC1?style=for-the-badge&logo=neo4j&logoColor=white)
![Data](https://img.shields.io/badge/Formats-PDF_DOCX_XLSX_CSV_HTML-2ea44f?style=for-the-badge)
![AI](https://img.shields.io/badge/AI-Hybrid_RAG-8A2BE2?style=for-the-badge&logo=openai)
![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)

> **Un sistema de Recuperación Aumentada (RAG) de alto rendimiento que fusiona la velocidad de Rust, la estructura de los Grafos de Conocimiento y el razonamiento de LLMs.**

---

**[ 🇪🇸 Español ](#-español) | [ 🇺🇸 English ](#-english) | [ 🏴󠁥󠁳󠁣󠁴󠁿 Català ](#-català)**

---

<a name="es"></a>
## 🇪🇸 Español

### 📖 Descripción
**LaMuralla** no es solo un chat con documentos; es un motor cognitivo de última generación. A diferencia de los sistemas RAG tradicionales, LaMuralla construye un **Grafo de Conocimiento** estructurado a partir de una amplia variedad de fuentes de datos.

El sistema incorpora un potente módulo de **Transmutación** que normaliza documentos complejos (hojas de cálculo, reportes legales, código web) en conocimiento puro, permitiendo entender no solo *qué* dicen los datos, sino *cómo* se conectan.

### ✨ Características Principales
*   **⚡ Core en Rust:** Backend construido sobre `Axum` y `Tokio` para una latencia mínima y seguridad de memoria.
*   **📄 Ingesta Universal de Datos:** Soporte nativo y robusto para múltiples formatos. El motor procesa, limpia y estructura automáticamente:
    *   **Documentos:** PDF, DOCX, TXT.
    *   **Datos Estructurados:** Excel (XLSX), CSV.
    *   **Web & Código:** HTML, JSON, XML, Markdown.
*   **🕸️ RAG Híbrido:** Combina búsqueda vectorial (Embeddings) con travesía de grafos (Cypher) para un contexto insuperable.
*   **🧠 Razonamiento Inferido:** Módulo de IA que analiza el grafo para descubrir y crear nuevas conexiones lógicas no explícitas en el texto original.
*   **👁️ Visualización Interactiva:** Interfaz profesional para explorar el conocimiento visualmente ("Deep Dive") y entender las relaciones entre entidades.

### 🛠️ Tech Stack
| Componente | Tecnología | Descripción |
| :--- | :--- | :--- |
| **Backend** | Rust (Axum) | API REST asíncrona de alto rendimiento. |
| **Parsing** | Calamine / Lopdf | Motor de "Transmutación" para Excel, PDF y más. |
| **Base de Datos** | Neo4j | Almacenamiento híbrido: Grafo nativo + Índice Vectorial. |
| **Orquestación IA** | Rig-Core | Framework de Rust para construir aplicaciones LLM. |
| **Frontend** | Tera / Bootstrap 5 | Interfaz SSR con renderizado dinámico y Vis.js. |

### 🚀 Instalación y Uso

#### 1. Configuración
Crea un archivo `.env` en la raíz con tus credenciales (Neo4j y OpenAI/Groq).

#### 2. Ejecución
**Modo Local:**
```bash
cargo run --release
```
**Modo Docker:**
```bash
docker build -t lamuralla-engine .
docker run -p 3000:3000 --env-file .env lamuralla-engine
```
Accede a la UI en: `http://localhost:3000`

---

<a name="en"></a>
## 🇺🇸 English

### 📖 Description
**LaMuralla** is more than just a chat-with-docs app; it is a next-gen cognitive engine. Unlike traditional RAG systems, LaMuralla constructs a structured **Knowledge Graph** from a wide array of data sources.

The system features a powerful **Transmutation** module that normalizes complex documents (spreadsheets, legal reports, web code) into pure knowledge, enabling the system to understand not just *what* the data says, but *how* it connects.

### ✨ Key Features
*   **⚡ Rust Core:** Backend built on `Axum` and `Tokio` for minimal latency and memory safety.
*   **📄 Universal Data Ingestion:** Robust native support for multiple formats. The engine automatically processes, cleans, and structures:
    *   **Documents:** PDF, DOCX, TXT.
    *   **Structured Data:** Excel (XLSX), CSV.
    *   **Web & Code:** HTML, JSON, XML, Markdown.
*   **🕸️ Hybrid RAG:** Combines vector search (Embeddings) with graph traversal (Cypher) for superior context retrieval.
*   **🧠 Inferred Reasoning:** AI module that analyzes the graph to discover and create new logical connections not explicitly stated in the source text.
*   **👁️ Interactive Visualization:** Professional UI to visually explore knowledge ("Deep Dive") and understand entity relationships.

### 🛠️ Tech Stack
| Component | Technology | Description |
| :--- | :--- | :--- |
| **Backend** | Rust (Axum) | High-performance asynchronous REST API. |
| **Parsing** | Calamine / Lopdf | "Transmutation" engine for Excel, PDF, and more. |
| **Database** | Neo4j | Hybrid storage: Native Graph + Vector Index. |
| **AI Orchestration** | Rig-Core | Rust framework for building LLM applications. |
| **Frontend** | Tera / Bootstrap 5 | SSR interface with dynamic rendering and Vis.js. |

### 🚀 Setup & Usage

#### 1. Configuration
Create a `.env` file in the root directory with your credentials (Neo4j and OpenAI/Groq).

#### 2. Running the App
**Local Mode:**
```bash
cargo run --release
```
**Docker Mode:**
```bash
docker build -t lamuralla-engine .
docker run -p 3000:3000 --env-file .env lamuralla-engine
```
Access the UI at: `http://localhost:3000`

---

<a name="ca"></a>
## 🏴󠁥󠁳󠁣󠁴󠁿 Català

### 📖 Descripció
**LaMuralla** no és només un xat amb documents; és un motor cognitiu d'última generació. A diferència dels sistemes RAG tradicionals, LaMuralla construeix un **Graf de Coneixement** estructurat a partir d'una àmplia varietat de fonts de dades.

El sistema incorpora un potent mòdul de **Transmutació** que normalitza documents complexos (fulls de càlcul, informes legals, codi web) en coneixement pur, permetent entendre no només *què* diuen les dades, sinó *com* es connecten.

### ✨ Característiques Principals
*   **⚡ Core en Rust:** Backend construït sobre `Axum` i `Tokio` per a una latència mínima i seguretat de memòria.
*   **📄 Ingesta Universal de Dades:** Suport natiu i robust per a múltiples formats. El motor processa, neteja i estructura automàticament:
    *   **Documents:** PDF, DOCX, TXT.
    *   **Dades Estructurades:** Excel (XLSX), CSV.
    *   **Web i Codi:** HTML, JSON, XML, Markdown.
*   **🕸️ RAG Híbrid:** Combina cerca vectorial (Embeddings) amb recorregut de grafs (Cypher) per a un context insuperable.
*   **🧠 Raonament Inferit:** Mòdul d'IA que analitza el graf per descobrir i crear noves connexions lògiques no explícites en el text original.
*   **👁️ Visualització Interactiva:** Interfície professional per explorar el coneixement visualment ("Deep Dive") i entendre les relacions entre entitats.

### 🛠️ Pila Tecnològica
| Component | Tecnologia | Descripció |
| :--- | :--- | :--- |
| **Backend** | Rust (Axum) | API REST asíncrona d'alt rendiment. |
| **Parsing** | Calamine / Lopdf | Motor de "Transmutació" per a Excel, PDF i més. |
| **Base de Dades** | Neo4j | Emmagatzematge híbrid: Graf natiu + Índex Vectorial. |
| **Orquestració IA** | Rig-Core | Framework de Rust per construir aplicacions LLM. |
| **Frontend** | Tera / Bootstrap 5 | Interfície SSR amb renderitzat dinàmic i Vis.js. |

### 🚀 Instal·lació i Ús

#### 1. Configuració
Crea un fitxer `.env` a l'arrel amb les teves credencials (Neo4j i OpenAI/Groq).

#### 2. Execució
**Mode Local:**
```bash
cargo run --release
```
**Mode Docker:**
```bash
docker build -t lamuralla-engine .
docker run -p 3000:3000 --env-file .env lamuralla-engine
```
Accedeix a la interfície a: `http://localhost:3000`

---

## 👨‍💻 Autor / Author

**Ángel A. Urbina**  
*Architecture & Development*

[![CV](https://img.shields.io/badge/Ver_Perfil_Profesional-000000?style=for-the-badge&logo=About.me&logoColor=white)](https://angelurbinacv.netlify.app/)

---

© 2025 LaMuralla Project. All Rights Reserved.

