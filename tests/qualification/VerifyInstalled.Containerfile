# Offline installed candidate; attests the exact RPM consumed by the native runner.
ARG BASE=localhost/memoquill-m1-desktop:20261027
FROM ${BASE}
USER root
COPY candidate.rpm /tmp/memoquill.rpm
COPY rpm.sha256 /opt/memoquill-verification-rpm.sha256
RUN rpm --install /tmp/memoquill.rpm && rm /tmp/memoquill.rpm
USER learner
