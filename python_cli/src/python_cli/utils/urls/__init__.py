"""Utilities for URLS."""


def merge_url_parts(part1: str, part2: str) -> str:
    """Merges two parts of a URL path, ensuring exactly one slash between them.

    Handles cases where:
    - part1 ends with '/' and part2 starts with '/' (removes duplicate)
    - part1 ends with '/' and part2 doesn't start with '/' (keeps single slash)
    - part1 doesn't end with '/' and part2 starts with '/' (keeps single slash)
    - part1 doesn't end with '/' and part2 doesn't start with '/' (adds single slash)
    - Either part is empty.

    Args:
      part1: The first part of the URL (e.g., 'http://example.com/api', 'path/to').
      part2: The second part of the URL (e.g., 'users', '/data', 'resource/id').

    Returns:
      The merged URL string with a single slash separator.
    """
    # Handle empty parts gracefully
    if not part1:
        # If part1 is empty, just return part2, ensuring it doesn't start with '//'
        # (though lstrip below handles this mostly)
        return (
            part2.lstrip("/") if part2 == "/" else part2
        )  # Avoid returning "" if part2 is "/"
    if not part2:
        # If part2 is empty, just return part1, ensuring it doesn't end with '//'
        # (though rstrip below handles this mostly)
        return part1

    # Strip trailing slash from part1 and leading slash from part2
    cleaned_part1 = part1.rstrip("/")
    cleaned_part2 = part2.lstrip("/")

    # Join them with a single slash
    return f"{cleaned_part1}/{cleaned_part2}"
