class WorkerError(Exception):
    """Base worker exception for ALIV subprocess workers."""


class WorkerValidationError(WorkerError):
    """Raised when an invalid worker payload is received."""
