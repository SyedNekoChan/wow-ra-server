#!/usr/bin/env bash
# Opens the world server console.
# Leave it by pressing Ctrl+X, then X.  NEVER press Ctrl+C (it kills the server).
docker attach --detach-keys="ctrl-x,x" ac-worldserver
