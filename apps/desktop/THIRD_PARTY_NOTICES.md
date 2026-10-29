# Third-party notices

ALIV Desktop is licensed under the Apache License, Version 2.0 (see [LICENSE](LICENSE)).

Parts of this application derive from open-source software developed by Menlo Research (https://menlo.ai), which is licensed under the Apache License, Version 2.0. Their copyright and attribution notices are kept in [LICENSE](LICENSE), as section 4 of that license requires.

Other open-source dependencies, and their licenses, are listed in `package.json`, `yarn.lock` and the `Cargo.toml`/`Cargo.lock` files. Notable ones include Tauri, React, llama.cpp, SQLite and pdf-extract.

## Vendored packages

`packages/streamdown` and `packages/remend` are the published builds of a modified version of [streamdown](https://github.com/vercel/streamdown) (Apache-2.0, originally by Vercel, Inc.), vendored with their license in each package directory.
