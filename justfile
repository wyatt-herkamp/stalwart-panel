#!/usr/bin/env -S just --justfile
set windows-shell := ["powershell.exe", "-c"]
project_root := justfile_directory()
panel_frontend := join(project_root,"panel-frontend")

format:
  cargo fmt --all
  cd {{panel_frontend}}; npm run format --write src/

update:
  echo "Updating the Cargo.lock"
  cargo update
  echo "Updating package-lock.json"
  cd {{panel_frontend}};  npm update
