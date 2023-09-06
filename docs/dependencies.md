# Internal crates interdependencies

## Overview

```mermaid
flowchart TB
    resolver --> logger 
    resolver --> semconv 
    resolver --> schema     
    schema --> semconv
    schema --> version
    
    main --> logger 
    main --> resolver
    
    Note right of version: handle telemetry schema versions.
```
