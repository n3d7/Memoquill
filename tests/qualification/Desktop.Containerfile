FROM localhost/memoquill-m1-qualification:20261027
USER root
RUN dnf -y --setopt=install_weak_deps=False install openbox /usr/libexec/ibus-ui-gtk3 && dnf clean all
USER learner
