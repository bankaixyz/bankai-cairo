#!/bin/bash

# Ensure the script is sourced so that environment changes persist.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "Error: This script must be sourced. Please run:"
    echo "  source scripts/activate.sh"
    exit 1
fi

# Use the current working directory as the project root.
PROJECT_ROOT=$(pwd)

# Check for the existence of the venv directory.
if [ ! -d "$PROJECT_ROOT/venv" ]; then
    echo "Error: 'venv' directory not found in current working directory ($PROJECT_ROOT)."
    return 1
fi

# Activate the virtual environment.
if [ -f "$PROJECT_ROOT/venv/bin/activate" ]; then
    source "$PROJECT_ROOT/venv/bin/activate"
else
    echo "Error: Virtual environment activation script not found at $PROJECT_ROOT/venv/bin/activate"
    return 1
fi

# Add the venv site-packages to PYTHONPATH.
VENV_SITE_PACKAGES="$PROJECT_ROOT/venv/lib/python3.10/site-packages"
export PYTHONPATH="$PYTHONPATH:$VENV_SITE_PACKAGES"

# Ensure embedded Python can locate the stdlib when PyO3 initializes.
if [ -z "${PYTHONHOME:-}" ]; then
    if [ -x "$PROJECT_ROOT/venv/bin/python3.10" ]; then
        PYTHONHOME="$("$PROJECT_ROOT/venv/bin/python3.10" -c 'import sys; print(sys.base_prefix)')"
    elif [ -x "$PROJECT_ROOT/venv/bin/python3" ]; then
        PYTHONHOME="$("$PROJECT_ROOT/venv/bin/python3" -c 'import sys; print(sys.base_prefix)')"
    elif [ -x "$PROJECT_ROOT/venv/bin/python" ]; then
        PYTHONHOME="$("$PROJECT_ROOT/venv/bin/python" -c 'import sys; print(sys.base_prefix)')"
    fi
    if [ -n "${PYTHONHOME:-}" ]; then
        export PYTHONHOME
    fi
fi

echo "Virtual environment activated and PYTHONPATH updated."
