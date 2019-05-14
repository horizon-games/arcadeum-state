#!/bin/sh

set -e

echo "= building bindings... ="
(set -x ; cd bindings ; yarn ; yarn build)
echo

echo "= building examples... ="
echo
for i in examples/* ; do
  echo "== building ${i#examples/} module... =="
  (set -x ; cd "$i/src-ts" ; yarn ; yarn build)
  echo

  echo "== building ${i#examples/} test... =="
  (set -x ; cd "$i/test" ; yarn ; yarn build)
  echo
done

echo "= building substrate... ="
(set -x ; cd substrate ; scripts/build.sh)
echo
