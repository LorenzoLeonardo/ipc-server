# ipc-server
An inter-process communication system that manages messages across processes via TCP stream.
It uses JSON format strings as a protocol when exchanging messages across processes.

This is also a library for the client-side processes for Rust.
The user application can share the object across the TCP stream.

## Overview of the IPC Server, registering objects and making remote calls
```mermaid
sequenceDiagram
    participant proc1 as Process 1 (Client Sharer)
    participant server as IPC Server 127.0.0.1:<PORT>
    participant proc2 as Process 2 (Client Requestor)

    proc1->>server: connect(127.0.0.1:<PORT>)
    proc1->>server: register_object {"reg_object": "object_name"}
    server->>server: register_object(object_name, client_socket)
    server-->>proc1: {"success":"OK"}

    proc2->>server: connect(127.0.0.1:<PORT>)
    proc2->>server: wait_for_objects {"list":["object_name"]}
    server->>server: find_registered_objects()
    server-->>proc2: {"list":["object_name"]}

    proc2->>server: remote_call {"object":"object name", "method":"method_name","param": JsonValue}
    server->>server: find_registered_objects()
    server->>proc1: {"object":"object name", "method":"method_name","param":JsonValue}
    proc1-->>server: {"response": JsonValue}
    server-->>proc2: {"response": JsonValue}
```

## Overview of the IPC Server, listening for events and sending events
```mermaid
sequenceDiagram
    participant proc1 as Process 1..n (Event Listeners)
    participant server as IPC Server 127.0.0.1:<PORT>
    participant proc2 as Process 2 (Event Sender)

    proc1->>server: connect(127.0.0.1:<PORT>)
    proc1->>server: listen_for_events {"event_name": "event"}

    server->>server: add_subscriber

    proc2->>server: connect(127.0.0.1:<PORT>)
    proc2->>server: send_event{"event":"event", "result": JsonValue}
    loop --> broadcast to subscribers

    server->>proc1: broad_cast_event {"event": "result": JsonValue}
    end
```