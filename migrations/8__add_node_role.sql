-- Migration: Add role column to nodes table
-- Valid values: 'head' | 'worker'
-- Default 'worker' keeps existing records consistent.

ALTER TABLE nodes ADD COLUMN role TEXT NOT NULL DEFAULT 'worker';
