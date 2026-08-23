#!/usr/bin/with-contenv bash
# Local port forwarding (what SshTunnel relies on) is disabled by this
# image's default sshd_config — enabled here for this throwaway test
# bastion only. Never do this to a real production sshd without knowing
# exactly what you're opening up.
#
# This runs after sshd has already started with the un-patched config
# (custom-cont-init.d scripts always run late), so the change is
# applied to the live config file and sshd is restarted to pick it up.
sed -i 's/^AllowTcpForwarding no/AllowTcpForwarding yes/' /config/sshd/sshd_config
s6-svc -r /run/service/svc-openssh-server
