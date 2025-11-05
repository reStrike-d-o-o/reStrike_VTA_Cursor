CREATE TABLE schema_version (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version INTEGER NOT NULL,
                applied_at TEXT NOT NULL,
                description TEXT NOT NULL
            );
CREATE TABLE sqlite_sequence(name,seq);
CREATE TABLE obs_connections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                password TEXT,
                is_active BOOLEAN NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            , status TEXT NOT NULL DEFAULT 'disconnected', error TEXT, created INTEGER, updated INTEGER);
CREATE TABLE app_config (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,
                value TEXT NOT NULL,
                category TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            , created INTEGER, updated INTEGER);
CREATE INDEX idx_app_config_category ON app_config(category);
CREATE TABLE flags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                ioc_code TEXT,
                country_name TEXT,
                recognition_status TEXT DEFAULT 'pending',
                recognition_confidence REAL,
                upload_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_modified DATETIME DEFAULT CURRENT_TIMESTAMP,
                file_size INTEGER,
                file_path TEXT NOT NULL,
                is_recognized BOOLEAN DEFAULT FALSE
            );
CREATE TABLE recognition_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                flag_id INTEGER,
                recognition_method TEXT,
                confidence REAL,
                recognized_as TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (flag_id) REFERENCES flags(id)
            );
CREATE INDEX idx_flags_ioc_code ON flags(ioc_code);
CREATE INDEX idx_flags_recognition_status ON flags(recognition_status);
CREATE INDEX idx_recognition_history_flag_id ON recognition_history(flag_id);
CREATE TABLE network_interfaces (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                address TEXT NOT NULL,
                netmask TEXT,
                broadcast TEXT,
                is_loopback BOOLEAN NOT NULL DEFAULT 0,
                is_active BOOLEAN NOT NULL DEFAULT 0,
                is_recommended BOOLEAN NOT NULL DEFAULT 0,
                speed_mbps INTEGER,
                mtu INTEGER,
                mac_address TEXT,
                interface_type TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL, created INTEGER, updated INTEGER,
                UNIQUE(name, address)
            );
CREATE TABLE udp_server_configs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                port INTEGER NOT NULL,
                bind_address TEXT NOT NULL,
                network_interface_id INTEGER,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                auto_start BOOLEAN NOT NULL DEFAULT 0,
                max_packet_size INTEGER DEFAULT 1024,
                buffer_size INTEGER DEFAULT 8192,
                timeout_ms INTEGER DEFAULT 1000,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL, created INTEGER, updated INTEGER,
                FOREIGN KEY (network_interface_id) REFERENCES network_interfaces(id)
            );
CREATE TABLE udp_server_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_config_id INTEGER NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                status TEXT NOT NULL DEFAULT 'running',
                packets_received INTEGER DEFAULT 0,
                packets_parsed INTEGER DEFAULT 0,
                parse_errors INTEGER DEFAULT 0,
                total_bytes_received INTEGER DEFAULT 0,
                average_packet_size REAL DEFAULT 0.0,
                max_packet_size_seen INTEGER DEFAULT 0,
                min_packet_size_seen INTEGER DEFAULT 0,
                unique_clients_count INTEGER DEFAULT 0,
                error_message TEXT, created INTEGER, updated INTEGER,
                FOREIGN KEY (server_config_id) REFERENCES udp_server_configs(id)
            );
CREATE TABLE udp_client_connections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                client_address TEXT NOT NULL,
                client_port INTEGER NOT NULL,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                packets_received INTEGER DEFAULT 0,
                total_bytes_received INTEGER DEFAULT 0,
                is_active BOOLEAN NOT NULL DEFAULT 1, created INTEGER,
                FOREIGN KEY (session_id) REFERENCES udp_server_sessions(id),
                UNIQUE(session_id, client_address, client_port)
            );
CREATE TABLE pss_event_types (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_code TEXT NOT NULL UNIQUE,
                event_name TEXT NOT NULL,
                description TEXT,
                category TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            , created INTEGER);
CREATE TABLE pss_athletes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                athlete_code TEXT NOT NULL UNIQUE,
                short_name TEXT NOT NULL,
                long_name TEXT,
                country_code TEXT,
                flag_id INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL, created INTEGER, updated INTEGER,
                FOREIGN KEY (flag_id) REFERENCES flags(id)
            );
CREATE TABLE pss_event_details (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id INTEGER NOT NULL,
                detail_key TEXT NOT NULL,
                detail_value TEXT,
                detail_type TEXT NOT NULL, -- string, integer, float, boolean, json
                created_at TEXT NOT NULL, created INTEGER,
                FOREIGN KEY (event_id) REFERENCES "pss_events"(id),
                UNIQUE(event_id, detail_key)
            );
CREATE INDEX idx_network_interfaces_active ON network_interfaces(is_active, is_recommended);
CREATE INDEX idx_udp_server_configs_enabled ON udp_server_configs(enabled);
CREATE INDEX idx_udp_server_sessions_status ON udp_server_sessions(status, start_time);
CREATE INDEX idx_udp_client_connections_session ON udp_client_connections(session_id, is_active);
CREATE INDEX idx_pss_event_details_event ON pss_event_details(event_id);
CREATE INDEX idx_pss_event_types_event_code ON pss_event_types(event_code);
CREATE INDEX idx_pss_event_types_category ON pss_event_types(category);
CREATE INDEX idx_pss_event_types_is_active ON pss_event_types(is_active);
CREATE INDEX idx_pss_athletes_athlete_code ON pss_athletes(athlete_code);
CREATE INDEX idx_pss_athletes_created_at ON pss_athletes(created_at);
CREATE INDEX idx_udp_server_configs_name ON udp_server_configs(name);
CREATE INDEX idx_udp_server_sessions_config_id ON udp_server_sessions(server_config_id);
CREATE INDEX idx_udp_server_sessions_start_time ON udp_server_sessions(start_time);
CREATE INDEX idx_udp_client_connections_session_id ON udp_client_connections(session_id);
CREATE INDEX idx_udp_client_connections_client_address ON udp_client_connections(client_address);
CREATE INDEX idx_udp_client_connections_first_seen ON udp_client_connections(first_seen);
CREATE INDEX idx_network_interfaces_name ON network_interfaces(name);
CREATE INDEX idx_network_interfaces_is_active ON network_interfaces(is_active);
CREATE INDEX idx_schema_version_version ON schema_version(version);
CREATE INDEX idx_schema_version_applied_at ON schema_version(applied_at);
CREATE TABLE pss_event_recognition_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id INTEGER NOT NULL,
                old_status TEXT NOT NULL,
                new_status TEXT NOT NULL,
                changed_by TEXT NOT NULL DEFAULT 'system',
                change_reason TEXT,
                protocol_version TEXT,
                raw_data TEXT NOT NULL,
                parsed_data TEXT,
                created_at TEXT NOT NULL, created INTEGER,
                FOREIGN KEY (event_id) REFERENCES "pss_events"(id) ON DELETE CASCADE
            );
CREATE TABLE pss_unknown_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                raw_data TEXT NOT NULL,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                occurrence_count INTEGER DEFAULT 1,
                pattern_hash TEXT,
                suggested_event_type TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL, created INTEGER, updated INTEGER,
                FOREIGN KEY (session_id) REFERENCES udp_server_sessions(id) ON DELETE CASCADE
            );
CREATE TABLE pss_event_validation_rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_code TEXT NOT NULL,
                protocol_version TEXT NOT NULL,
                rule_name TEXT NOT NULL,
                rule_type TEXT NOT NULL CHECK (rule_type IN ('format', 'data_type', 'range', 'required', 'custom')),
                rule_definition TEXT NOT NULL,
                error_message TEXT,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL, created INTEGER, updated INTEGER,
                UNIQUE(event_code, protocol_version, rule_name)
            );
CREATE TABLE pss_event_validation_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id INTEGER NOT NULL,
                rule_id INTEGER NOT NULL,
                validation_passed BOOLEAN NOT NULL,
                error_message TEXT,
                validation_time_ms INTEGER,
                created_at TEXT NOT NULL, created INTEGER,
                FOREIGN KEY (event_id) REFERENCES "pss_events"(id) ON DELETE CASCADE,
                FOREIGN KEY (rule_id) REFERENCES pss_event_validation_rules(id) ON DELETE CASCADE
            );
CREATE TABLE pss_event_statistics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                event_type_id INTEGER,
                total_events INTEGER DEFAULT 0,
                recognized_events INTEGER DEFAULT 0,
                unknown_events INTEGER DEFAULT 0,
                partial_events INTEGER DEFAULT 0,
                deprecated_events INTEGER DEFAULT 0,
                validation_errors INTEGER DEFAULT 0,
                parsing_errors INTEGER DEFAULT 0,
                average_processing_time_ms REAL DEFAULT 0.0,
                min_processing_time_ms INTEGER,
                max_processing_time_ms INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL, created INTEGER, updated INTEGER,
                FOREIGN KEY (session_id) REFERENCES udp_server_sessions(id) ON DELETE CASCADE,
                FOREIGN KEY (event_type_id) REFERENCES pss_event_types(id) ON DELETE SET NULL
            );
CREATE INDEX idx_pss_event_recognition_history_event_id ON pss_event_recognition_history(event_id);
CREATE INDEX idx_pss_event_recognition_history_status_change ON pss_event_recognition_history(old_status, new_status);
CREATE INDEX idx_pss_event_recognition_history_created_at ON pss_event_recognition_history(created_at);
CREATE INDEX idx_pss_unknown_events_session_id ON pss_unknown_events(session_id);
CREATE INDEX idx_pss_unknown_events_pattern_hash ON pss_unknown_events(pattern_hash);
CREATE INDEX idx_pss_unknown_events_first_seen ON pss_unknown_events(first_seen);
CREATE INDEX idx_pss_event_validation_rules_event_code ON pss_event_validation_rules(event_code);
CREATE INDEX idx_pss_event_validation_rules_protocol_version ON pss_event_validation_rules(protocol_version);
CREATE INDEX idx_pss_event_validation_rules_active ON pss_event_validation_rules(is_active);
CREATE INDEX idx_pss_event_validation_results_event_id ON pss_event_validation_results(event_id);
CREATE INDEX idx_pss_event_validation_results_rule_id ON pss_event_validation_results(rule_id);
CREATE INDEX idx_pss_event_validation_results_passed ON pss_event_validation_results(validation_passed);
CREATE INDEX idx_pss_event_statistics_session_id ON pss_event_statistics(session_id);
CREATE INDEX idx_pss_event_statistics_event_type_id ON pss_event_statistics(event_type_id);
CREATE TABLE obs_scenes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scene_name TEXT NOT NULL UNIQUE,
                scene_id TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                last_seen_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            , created INTEGER, updated INTEGER);
CREATE TABLE overlay_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                theme TEXT NOT NULL DEFAULT 'default',
                colors TEXT, -- JSON string for color configuration
                animation_type TEXT NOT NULL DEFAULT 'fade',
                duration_ms INTEGER NOT NULL DEFAULT 3000,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            , url TEXT, created INTEGER, updated INTEGER);
CREATE INDEX idx_obs_scenes_active ON obs_scenes(is_active);
CREATE INDEX idx_obs_scenes_name ON obs_scenes(scene_name);
CREATE INDEX idx_overlay_templates_active ON overlay_templates(is_active);
CREATE INDEX idx_overlay_templates_theme ON overlay_templates(theme);
CREATE TABLE secure_config (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                config_key TEXT NOT NULL UNIQUE,
                encrypted_value BLOB NOT NULL,
                category TEXT NOT NULL,
                is_sensitive BOOLEAN NOT NULL DEFAULT 1,
                salt BLOB NOT NULL,
                algorithm TEXT NOT NULL DEFAULT 'AES-256-GCM',
                kdf_params TEXT NOT NULL, -- JSON with KDF parameters
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_accessed TEXT,
                access_count INTEGER DEFAULT 0,
                description TEXT
            );
CREATE UNIQUE INDEX idx_secure_config_key ON secure_config(config_key);
CREATE INDEX idx_secure_config_category ON secure_config(category);
CREATE INDEX idx_secure_config_sensitive ON secure_config(is_sensitive);
CREATE TABLE config_audit (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                config_key TEXT NOT NULL,
                action TEXT NOT NULL, -- 'create', 'read', 'update', 'delete'
                user_context TEXT, -- User or system context
                source_ip TEXT, -- IP address if applicable
                timestamp TEXT NOT NULL,
                details TEXT, -- Additional audit details as JSON
                success BOOLEAN NOT NULL DEFAULT 1,
                error_message TEXT
            );
CREATE INDEX idx_config_audit_key ON config_audit(config_key);
CREATE INDEX idx_config_audit_action ON config_audit(action);
CREATE INDEX idx_config_audit_timestamp ON config_audit(timestamp);
CREATE TABLE security_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL UNIQUE,
                user_context TEXT NOT NULL,
                access_level TEXT NOT NULL, -- 'read_only', 'configuration', 'administrator'
                created_at TEXT NOT NULL,
                last_accessed TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                source_ip TEXT,
                user_agent TEXT
            );
CREATE UNIQUE INDEX idx_security_sessions_id ON security_sessions(session_id);
CREATE INDEX idx_security_sessions_expires ON security_sessions(expires_at);
CREATE TABLE config_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category_name TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                description TEXT,
                access_level TEXT NOT NULL DEFAULT 'configuration', -- Required access level
                is_system BOOLEAN NOT NULL DEFAULT 0, -- System vs user category
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
CREATE TABLE IF NOT EXISTS "obs_recording_config" (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                obs_connection_name TEXT NOT NULL UNIQUE,
                recording_root_path TEXT NOT NULL,
                recording_format TEXT NOT NULL DEFAULT 'mp4',
                replay_buffer_enabled BOOLEAN NOT NULL DEFAULT 1,
                replay_buffer_duration INTEGER DEFAULT 30,
                auto_start_recording BOOLEAN NOT NULL DEFAULT 1,
                auto_start_replay_buffer BOOLEAN NOT NULL DEFAULT 1,
                filename_template TEXT NOT NULL DEFAULT '{matchNumber}_{player1}_{player2}_{date}',
                folder_pattern TEXT NOT NULL DEFAULT '{tournament}/{tournamentDay}',
                is_active BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
CREATE TABLE recorded_video_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                recorded_video_id INTEGER NOT NULL,
                event_id INTEGER NOT NULL,
                offset_ms INTEGER NOT NULL,
                created_at TEXT NOT NULL, created INTEGER,
                UNIQUE(recorded_video_id, event_id),
                FOREIGN KEY (recorded_video_id) REFERENCES recorded_videos(id) ON DELETE CASCADE,
                FOREIGN KEY (event_id) REFERENCES "pss_events"(id) ON DELETE CASCADE
            );
CREATE INDEX idx_rve_video ON recorded_video_events(recorded_video_id);
CREATE INDEX idx_rve_event ON recorded_video_events(event_id);
CREATE TABLE look_genders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
CREATE TABLE look_disciplines (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
CREATE TABLE look_age_groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                min_age INTEGER,
                max_age INTEGER,
                authority TEXT,
                effective_from TEXT,
                effective_to TEXT,
                created_at TEXT NOT NULL
            );
CREATE TABLE look_divisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                division_type TEXT, -- belt|poomsae|other
                authority TEXT,
                created_at TEXT NOT NULL
            );
CREATE TABLE look_weight_classes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                discipline_id INTEGER NOT NULL,
                gender_id INTEGER NOT NULL,
                age_group_id INTEGER NOT NULL,
                code TEXT NOT NULL,
                name TEXT NOT NULL,
                min_kg REAL,
                max_kg REAL,
                authority TEXT,
                effective_from TEXT,
                effective_to TEXT,
                created_at TEXT NOT NULL,
                UNIQUE(discipline_id, gender_id, age_group_id, code),
                FOREIGN KEY (discipline_id) REFERENCES look_disciplines(id),
                FOREIGN KEY (gender_id) REFERENCES look_genders(id),
                FOREIGN KEY (age_group_id) REFERENCES look_age_groups(id)
            );
CREATE TABLE look_round_configs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                rounds INTEGER NOT NULL,
                round_duration INTEGER,
                rest_duration INTEGER,
                golden_round BOOLEAN NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            , golden_round_duration INTEGER, kyeshi_duration INTEGER);
CREATE INDEX idx_lg_code ON look_genders(code);
CREATE INDEX idx_ld_code ON look_disciplines(code);
CREATE INDEX idx_lag_code ON look_age_groups(code);
CREATE INDEX idx_ldv_code ON look_divisions(code);
CREATE INDEX idx_lwc_scopes ON look_weight_classes(discipline_id, gender_id, age_group_id);
CREATE TABLE ovr_providers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                base_url TEXT,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                rate_limit_ms INTEGER NOT NULL DEFAULT 1000,
                last_refreshed_at TEXT,
                last_status TEXT,
                last_error TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            , created INTEGER, updated INTEGER);
CREATE TABLE ovr_tournaments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                provider_id INTEGER NOT NULL,
                provider_tournament_id TEXT NOT NULL,
                name TEXT NOT NULL,
                start_date TEXT,
                end_date TEXT,
                city TEXT,
                country TEXT,
                url TEXT,
                status TEXT,
                last_seen_at TEXT,
                hash TEXT,
                etag TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL, created INTEGER, updated INTEGER,
                UNIQUE(provider_id, provider_tournament_id),
                FOREIGN KEY (provider_id) REFERENCES ovr_providers(id) ON DELETE CASCADE
            );
CREATE TABLE ovr_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tournament_id INTEGER NOT NULL,
                discipline TEXT,
                age_group TEXT,
                gender TEXT,
                division TEXT,
                weight_class TEXT,
                bracket_stage TEXT,
                provider_raw TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL, created INTEGER, updated INTEGER,
                FOREIGN KEY (tournament_id) REFERENCES ovr_tournaments(id) ON DELETE CASCADE
            );
CREATE TABLE ovr_to_local_tournament (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ovr_tournament_id INTEGER NOT NULL UNIQUE,
                local_tournament_id INTEGER NOT NULL,
                created_at TEXT NOT NULL, created INTEGER,
                FOREIGN KEY (ovr_tournament_id) REFERENCES ovr_tournaments(id) ON DELETE CASCADE,
                FOREIGN KEY (local_tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE
            );
CREATE INDEX idx_ovr_tournaments_provider ON ovr_tournaments(provider_id);
CREATE INDEX idx_ovr_tournaments_name ON ovr_tournaments(name);
CREATE INDEX idx_ovr_tournaments_start ON ovr_tournaments(start_date);
CREATE INDEX idx_ovr_categories_tournament ON ovr_categories(tournament_id);
CREATE TABLE IF NOT EXISTS "settings_categories" (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                display_order INTEGER DEFAULT 0,
                created INTEGER
            );
CREATE TABLE IF NOT EXISTS "settings_keys" (
                id TEXT PRIMARY KEY,
                category_id TEXT NOT NULL,
                key_name TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                description TEXT,
                data_type TEXT NOT NULL,
                default_value TEXT,
                validation_rules TEXT,
                is_required BOOLEAN DEFAULT 0,
                is_sensitive BOOLEAN DEFAULT 0,
                created INTEGER
            );
CREATE TABLE IF NOT EXISTS "settings_values" (
                id TEXT PRIMARY KEY,
                key_id TEXT NOT NULL,
                value TEXT NOT NULL,
                created INTEGER,
                updated INTEGER
            );
CREATE TABLE IF NOT EXISTS "settings_history" (
                id TEXT PRIMARY KEY,
                key_id TEXT NOT NULL,
                old_value TEXT,
                new_value TEXT,
                changed_by TEXT NOT NULL,
                change_reason TEXT,
                created INTEGER
            );
CREATE INDEX idx_settings_keys_category ON settings_keys(category_id);
CREATE INDEX idx_settings_values_key ON settings_values(key_id);
CREATE INDEX idx_settings_history_key ON settings_history(key_id);
CREATE TABLE IF NOT EXISTS "flag_mappings" (
                id TEXT PRIMARY KEY,
                pss_code TEXT,
                ioc_code TEXT,
                country_name TEXT,
                is_custom BOOLEAN DEFAULT 0,
                created INTEGER,
                updated INTEGER
            );
CREATE TABLE pss_rounds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id TEXT,
                round_number INTEGER,
                start_time TEXT,
                end_time TEXT,
                duration INTEGER,
                winner_athlete_position INTEGER,
                created_at TEXT
            );
CREATE INDEX idx_pss_rounds_match_id ON pss_rounds(match_id);
CREATE TABLE pss_match_athletes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id TEXT,
                athlete_id INTEGER,
                athlete_position INTEGER,
                bg_color TEXT,
                fg_color TEXT,
                created_at TEXT
            );
CREATE INDEX idx_pss_match_athletes_match_id ON pss_match_athletes(match_id);
CREATE TABLE pss_matches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT UNIQUE,
                tournament_id TEXT,
                match_id TEXT NOT NULL,
                match_number TEXT,
                category TEXT,
                weight_class TEXT,
                division TEXT,
                total_rounds INTEGER,
                round_duration INTEGER,
                countdown_type TEXT,
                format_type INTEGER,
                creation_mode TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                created INTEGER,
                updated INTEGER
            );
CREATE INDEX idx_pss_matches_tournament_id ON pss_matches(tournament_id);
CREATE TABLE pss_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                match_id INTEGER,
                round_id INTEGER,
                event_type_id INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                raw_data TEXT NOT NULL,
                parsed_data TEXT,
                event_sequence INTEGER,
                processing_time_ms INTEGER,
                is_valid BOOLEAN NOT NULL,
                error_message TEXT,
                recognition_status TEXT NOT NULL,
                protocol_version TEXT,
                parser_confidence REAL,
                validation_errors TEXT,
                tournament_id TEXT,
                created_at TEXT NOT NULL,
                created INTEGER
            );
CREATE INDEX idx_pss_events_tournament_id ON pss_events(tournament_id);
CREATE TABLE pss_scores (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id TEXT,
                round_id INTEGER,
                athlete_position INTEGER,
                score_type TEXT,
                score_value INTEGER,
                timestamp TEXT,
                tournament_id TEXT,
                created_at TEXT,
                created INTEGER
            );
CREATE TABLE pss_warnings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id TEXT,
                round_id INTEGER,
                athlete_position INTEGER,
                warning_type TEXT,
                warning_count INTEGER,
                timestamp TEXT,
                tournament_id TEXT,
                created_at TEXT,
                created INTEGER
            );
CREATE TABLE recorded_videos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id INTEGER NOT NULL,
                event_id INTEGER,
                tournament_id TEXT,
                video_type TEXT NOT NULL,
                file_path TEXT,
                record_directory TEXT,
                filename_formatting TEXT,
                start_time TEXT NOT NULL,
                duration_seconds INTEGER,
                file_size INTEGER,
                checksum TEXT,
                created_at TEXT NOT NULL,
                created INTEGER
            );
CREATE TABLE event_triggers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tournament_id INTEGER,
                event_type TEXT NOT NULL,
                trigger_type TEXT NOT NULL,
                obs_scene_id INTEGER,
                overlay_template_id INTEGER,
                is_enabled BOOLEAN NOT NULL,
                priority INTEGER NOT NULL,
                action TEXT,
                target_type TEXT,
                delay_ms INTEGER,
                action_kind TEXT,
                obs_connection_name TEXT,
                condition_round INTEGER,
                condition_once_per TEXT,
                debounce_ms INTEGER,
                cooldown_ms INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
CREATE TABLE obs_recording_sessions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    obs_connection_name TEXT NOT NULL,
                    tournament_id INTEGER,
                    match_id TEXT,
                    match_number TEXT,
                    player1_name TEXT,
                    player1_flag TEXT,
                    player2_name TEXT,
                    player2_flag TEXT,
                    recording_path TEXT NOT NULL,
                    recording_filename TEXT NOT NULL,
                    recording_start_time TEXT,
                    recording_end_time TEXT,
                    recording_duration INTEGER,
                    recording_size_bytes INTEGER,
                    replay_buffer_start_time TEXT,
                    replay_buffer_end_time TEXT,
                    replay_buffer_saved BOOLEAN NOT NULL DEFAULT 0,
                    replay_buffer_filename TEXT,
                    status TEXT NOT NULL DEFAULT 'pending',
                    error_message TEXT,
                    created INTEGER NOT NULL DEFAULT (strftime('%s','now')),
                    updated INTEGER NOT NULL DEFAULT (strftime('%s','now'))
                );
CREATE INDEX idx_obs_recording_sessions_created_int ON obs_recording_sessions(created);
CREATE TABLE medal_ceremonies (
                id TEXT PRIMARY KEY,
                tournament_id INTEGER,
                name TEXT NOT NULL,
                background_path TEXT,
                break_path TEXT,
                animation_duration INTEGER NOT NULL DEFAULT 45000,
                animation_speed REAL NOT NULL DEFAULT 1.0,
                photo_time INTEGER NOT NULL DEFAULT 10,
                prepared_at TEXT,
                prepared_version INTEGER NOT NULL DEFAULT 0,
                show_external INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE SET NULL
            );
CREATE TABLE medal_ceremony_divisions (
                id TEXT PRIMARY KEY,
                ceremony_id TEXT NOT NULL,
                division_id INTEGER,
                division_name TEXT NOT NULL,
                order_index INTEGER NOT NULL,
                played_at TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (ceremony_id) REFERENCES medal_ceremonies(id) ON DELETE CASCADE,
                FOREIGN KEY (division_id) REFERENCES look_divisions(id) ON DELETE SET NULL,
                UNIQUE(ceremony_id, order_index)
            );
CREATE TABLE medal_ceremony_medalists (
                id TEXT PRIMARY KEY,
                division_entry_id TEXT NOT NULL,
                medal_type TEXT NOT NULL,
                medal_rank INTEGER NOT NULL,
                athlete_id INTEGER,
                athlete_name TEXT NOT NULL,
                athlete_short_name TEXT,
                ioc_code TEXT,
                flag_asset TEXT,
                anthem_asset TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (division_entry_id) REFERENCES medal_ceremony_divisions(id) ON DELETE CASCADE,
                FOREIGN KEY (athlete_id) REFERENCES pss_athletes(id) ON DELETE SET NULL,
                UNIQUE(division_entry_id, medal_type)
            );
CREATE TABLE ovr_flag_animations (
                id TEXT PRIMARY KEY,
                ioc_code TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                display_name TEXT,
                duration_ms INTEGER,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
CREATE TABLE ovr_anthems (
                id TEXT PRIMARY KEY,
                ioc_code TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                display_name TEXT,
                duration_ms INTEGER,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
CREATE INDEX idx_medal_ceremony_divisions_ceremony_order
             ON medal_ceremony_divisions(ceremony_id, order_index);
CREATE INDEX idx_medal_ceremony_medalists_division
             ON medal_ceremony_medalists(division_entry_id);
CREATE INDEX idx_medal_ceremony_medalists_athlete
             ON medal_ceremony_medalists(athlete_id);
CREATE INDEX idx_ovr_flag_animations_ioc
             ON ovr_flag_animations(ioc_code);
CREATE UNIQUE INDEX idx_ovr_flag_animations_unique
             ON ovr_flag_animations(ioc_code, file_name);
CREATE INDEX idx_ovr_anthems_ioc
             ON ovr_anthems(ioc_code);
CREATE UNIQUE INDEX idx_ovr_anthems_unique
             ON ovr_anthems(ioc_code, file_name);
CREATE TABLE animations (
                id TEXT PRIMARY KEY,
                ioc_code TEXT,
                country_name TEXT,
                file_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                filename TEXT,
                display_name TEXT,
                duration_ms INTEGER,
                is_default INTEGER NOT NULL DEFAULT 0,
                recognition_status TEXT DEFAULT 'pending',
                recognition_confidence REAL,
                upload_date TEXT DEFAULT CURRENT_TIMESTAMP,
                last_modified TEXT DEFAULT CURRENT_TIMESTAMP,
                file_size INTEGER,
                is_recognized INTEGER DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
CREATE INDEX idx_animations_ioc ON animations(ioc_code);
CREATE UNIQUE INDEX idx_animations_unique
             ON animations(ioc_code, file_name);
CREATE TABLE tournament_rankings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                label TEXT NOT NULL,
                is_para INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
CREATE TABLE IF NOT EXISTS "tournaments" (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    uuid TEXT UNIQUE,
                    name TEXT NOT NULL UNIQUE,
                    duration_days INTEGER NOT NULL DEFAULT 1,
                    city TEXT NOT NULL,
                    country TEXT NOT NULL,
                    country_code TEXT,
                    logo_path TEXT,
                    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','running','ended')),
                    start_date TEXT,
                    end_date TEXT,
                    ranking_id INTEGER,
                    location TEXT NOT NULL DEFAULT '{}',
                    contact TEXT NOT NULL DEFAULT '{}',
                    oc TEXT NOT NULL DEFAULT '{}',
                    officials TEXT NOT NULL DEFAULT '{}',
                    banner TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    created INTEGER,
                    updated INTEGER,
                    FOREIGN KEY(ranking_id) REFERENCES tournament_rankings(id)
                );
CREATE INDEX idx_tournaments_status ON tournaments(status);
CREATE INDEX idx_tournaments_city_country ON tournaments(city, country);
CREATE INDEX idx_tournaments_name ON tournaments(name);
CREATE INDEX idx_tournaments_start_date ON tournaments(start_date);
CREATE INDEX idx_tournaments_created_at ON tournaments(created_at);
CREATE UNIQUE INDEX idx_tournaments_uuid ON tournaments(uuid);
CREATE TABLE tournament_days (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT UNIQUE,
                tournament_id INTEGER NOT NULL,
                day_number INTEGER NOT NULL,
                date TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','running','ended')),
                start_time TEXT,
                end_time TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                created INTEGER,
                updated INTEGER,
                FOREIGN KEY(tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE
            );
CREATE INDEX idx_tournament_days_tournament_id
             ON tournament_days(tournament_id);
CREATE INDEX idx_tournament_days_status
             ON tournament_days(status);
CREATE UNIQUE INDEX idx_tournament_days_unique
             ON tournament_days(tournament_id, date);
CREATE TABLE octagons (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tournament_id INTEGER NOT NULL,
                tournament_day_id INTEGER NOT NULL,
                octagon_number TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE,
                FOREIGN KEY(tournament_day_id) REFERENCES tournament_days(id) ON DELETE CASCADE
            );
CREATE UNIQUE INDEX idx_octagons_day_number
             ON octagons(tournament_day_id, octagon_number);
CREATE INDEX idx_octagons_tournament
             ON octagons(tournament_id);
CREATE TABLE athletes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                wtid TEXT,
                look_age_group_id INTEGER,
                look_division_id INTEGER,
                look_gender_id INTEGER,
                look_weight_class_id INTEGER,
                first_name TEXT,
                last_name TEXT,
                display_name TEXT,
                image TEXT,
                history TEXT NOT NULL DEFAULT '[]',
                country TEXT,
                country_code TEXT,
                ioc_code TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(look_age_group_id) REFERENCES look_age_groups(id) ON DELETE SET NULL,
                FOREIGN KEY(look_division_id) REFERENCES look_divisions(id) ON DELETE SET NULL,
                FOREIGN KEY(look_gender_id) REFERENCES look_genders(id) ON DELETE SET NULL,
                FOREIGN KEY(look_weight_class_id) REFERENCES look_weight_classes(id) ON DELETE SET NULL
            );
CREATE UNIQUE INDEX idx_athletes_wtid
             ON athletes(wtid) WHERE wtid IS NOT NULL;
CREATE INDEX idx_athletes_ioc_display
             ON athletes(ioc_code, display_name);
CREATE TABLE tournament_champions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tournament_uuid TEXT NOT NULL,
            category TEXT,
            match_uuid TEXT NOT NULL,
            match_id TEXT NOT NULL,
            winner_color TEXT NOT NULL,
            winner_name TEXT,
            winner_country_code TEXT,
            blue_score INTEGER NOT NULL,
            red_score INTEGER NOT NULL,
            medal_type TEXT NOT NULL DEFAULT 'gold',
            medal_rank INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
CREATE INDEX idx_tournament_champions_tournament ON tournament_champions(tournament_uuid);
CREATE INDEX idx_tournament_champions_category_rank ON tournament_champions(tournament_uuid, category, medal_rank);
