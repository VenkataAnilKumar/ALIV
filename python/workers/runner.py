from __future__ import annotations

import json
import sys
from typing import Callable

from workers.protocol import WorkerRequest, WorkerResponse

Handler = Callable[[WorkerRequest], WorkerResponse]


def run(handler: Handler) -> None:
    for raw_line in sys.stdin:
        raw_line = raw_line.strip()
        if not raw_line:
            continue
        request = WorkerRequest.model_validate_json(raw_line)
        response = handler(request)
        sys.stdout.write(json.dumps(response.model_dump()) + "\n")
        sys.stdout.flush()
