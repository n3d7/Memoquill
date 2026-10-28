FROM localhost/memoquill-m1-desktop:20261027
USER root
COPY Memoquill-0.1.0-1.x86_64.rpm /tmp/memoquill.rpm
RUN rpm --install /tmp/memoquill.rpm && rm /tmp/memoquill.rpm
USER learner
