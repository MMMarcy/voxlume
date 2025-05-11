"""Module containing the custom type definition.

These definitions will be used for *typed* dependency injection.
"""

from pathlib import Path
from typing import NewType

ConfigurationPath = NewType("ConfigurationPath", Path)
