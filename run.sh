#!/bin/bash
case "$1" in
  earth)        exec ./earth.sh ;;
  solar-system) exec ./solar-system.sh ;;
  male|car|vader|suitcase|canon) exec ./view-mesh.sh "$1" ;;
  *)
    echo ""
    echo "=== TermGL Demo Container ==="
    echo "  docker run -it --rm termgl earth"
    echo "  docker run -it --rm termgl solar-system"
    echo "  docker run -it --rm termgl male"
    echo "  docker run -it --rm termgl car"
    echo "  docker run -it --rm termgl vader"
    echo "  docker run -it --rm termgl suitcase"
    echo "  docker run -it --rm termgl canon"
    echo ""
    exec bash
    ;;
esac
