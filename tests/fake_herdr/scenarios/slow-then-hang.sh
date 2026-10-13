# pane get answers pane_not_found after 1.5 s, then workspace list hangs (the 3 s budget, AS-11).
case "$*" in
  "pane get w4V:p9") slow 1.5; fail 1 '{"error":{"code":"pane_not_found","message":"pane w4V:p9 not found"},"id":"cli:pane:get"}' ;;
  "workspace list") hang ;;
esac
