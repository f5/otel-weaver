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
    
    click resolver href "https://gitswarm.f5net.com/otel/weaver/-/tree/main/crates/resolver" _blank
    click logger href "https://gitswarm.f5net.com/otel/weaver/-/tree/main/crates/logger" _top
    click semconv "../crates/semconv"
    click schema "../crates/schema"
    click version "../crates/version"
    click main "../src"
```
