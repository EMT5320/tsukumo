CREATE TABLE handoff_checkpoints (
    checkpoint_id TEXT PRIMARY KEY NOT NULL,
    quest_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    previous_checkpoint_id TEXT,
    created_at INTEGER NOT NULL,
    created_event_id TEXT UNIQUE NOT NULL,
    checkpoint_json TEXT NOT NULL,
    UNIQUE(quest_id, version),
    FOREIGN KEY(previous_checkpoint_id) REFERENCES handoff_checkpoints(checkpoint_id),
    FOREIGN KEY(created_event_id) REFERENCES chronicle_events(event_id)
);
CREATE TABLE checkpoint_state_refs (
    checkpoint_id TEXT NOT NULL,
    state_id TEXT NOT NULL,
    state_version INTEGER NOT NULL,
    position INTEGER NOT NULL,
    PRIMARY KEY(checkpoint_id, state_id),
    FOREIGN KEY(checkpoint_id) REFERENCES handoff_checkpoints(checkpoint_id),
    FOREIGN KEY(state_id) REFERENCES state_records(state_id)
);
CREATE TABLE checkpoint_source_refs (
    checkpoint_id TEXT NOT NULL,
    event_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    PRIMARY KEY(checkpoint_id, event_id),
    FOREIGN KEY(checkpoint_id) REFERENCES handoff_checkpoints(checkpoint_id),
    FOREIGN KEY(event_id) REFERENCES chronicle_events(event_id)
);
CREATE TRIGGER handoff_checkpoints_no_update BEFORE UPDATE ON handoff_checkpoints BEGIN
    SELECT RAISE(ABORT, 'handoff_checkpoints is immutable');
END;
CREATE TRIGGER handoff_checkpoints_no_delete BEFORE DELETE ON handoff_checkpoints BEGIN
    SELECT RAISE(ABORT, 'handoff_checkpoints is immutable');
END;

CREATE TABLE projection_receipts (
    projection_id TEXT PRIMARY KEY NOT NULL,
    checkpoint_id TEXT NOT NULL,
    execution_id TEXT NOT NULL,
    runtime_json TEXT NOT NULL,
    projection_version INTEGER NOT NULL,
    renderer_version INTEGER NOT NULL,
    rendered_digest TEXT NOT NULL,
    rendered_byte_count INTEGER NOT NULL,
    rendered_char_count INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    created_event_id TEXT UNIQUE NOT NULL,
    receipt_json TEXT NOT NULL,
    FOREIGN KEY(checkpoint_id) REFERENCES handoff_checkpoints(checkpoint_id),
    FOREIGN KEY(created_event_id) REFERENCES chronicle_events(event_id)
);
CREATE TABLE receipt_state_refs (
    projection_id TEXT NOT NULL,
    state_id TEXT NOT NULL,
    state_version INTEGER NOT NULL,
    position INTEGER NOT NULL,
    PRIMARY KEY(projection_id, state_id),
    FOREIGN KEY(projection_id) REFERENCES projection_receipts(projection_id),
    FOREIGN KEY(state_id) REFERENCES state_records(state_id)
);
CREATE TRIGGER projection_receipts_no_update BEFORE UPDATE ON projection_receipts BEGIN
    SELECT RAISE(ABORT, 'projection_receipts is immutable');
END;
CREATE TRIGGER projection_receipts_no_delete BEFORE DELETE ON projection_receipts BEGIN
    SELECT RAISE(ABORT, 'projection_receipts is immutable');
END;
-- Checkpoint and receipt edges are immutable evidence, including their order.
CREATE TRIGGER checkpoint_state_refs_no_update BEFORE UPDATE ON checkpoint_state_refs BEGIN
    SELECT RAISE(ABORT, 'checkpoint_state_refs is immutable');
END;
CREATE TRIGGER checkpoint_state_refs_no_delete BEFORE DELETE ON checkpoint_state_refs BEGIN
    SELECT RAISE(ABORT, 'checkpoint_state_refs is immutable');
END;
CREATE TRIGGER checkpoint_source_refs_no_update BEFORE UPDATE ON checkpoint_source_refs BEGIN
    SELECT RAISE(ABORT, 'checkpoint_source_refs is immutable');
END;
CREATE TRIGGER checkpoint_source_refs_no_delete BEFORE DELETE ON checkpoint_source_refs BEGIN
    SELECT RAISE(ABORT, 'checkpoint_source_refs is immutable');
END;
CREATE TRIGGER receipt_state_refs_no_update BEFORE UPDATE ON receipt_state_refs BEGIN
    SELECT RAISE(ABORT, 'receipt_state_refs is immutable');
END;
CREATE TRIGGER receipt_state_refs_no_delete BEFORE DELETE ON receipt_state_refs BEGIN
    SELECT RAISE(ABORT, 'receipt_state_refs is immutable');
END;