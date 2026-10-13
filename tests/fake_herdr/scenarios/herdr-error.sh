# herdr fails with an error other than pane_not_found.
case "$*" in
  "pane get w4V:p9") fail 1 '{"error":{"code":"internal","message":"socket closed"}}' ;;
esac
