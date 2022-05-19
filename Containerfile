FROM openjdk:8-jdk

RUN apt update -y && apt install -y \
    build-essential git wget curl maven \
    bison flex python2 python3-matplotlib

ENV RUSTUP_HOME=/opt/rust \
    CARGO_HOME=/opt/rust

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --no-modify-path

RUN wget https://people.csail.mit.edu/asolar/sketch-1.7.6.tar.gz && \
    tar xvf sketch-1.7.6.tar.gz ; \
    rm sketch-1.7.6.tar.gz && \
    mv sketch-1.7.6 /opt/sketch && \
    cd /opt/sketch/sketch-backend && \
    ./configure && \
    make

RUN git clone https://github.com/diffblue/cbmc /opt/cbmc && \
    cd /opt/cbmc && make -C src DOWNLOADER=wget minisat2-download && \
    make -C jbmc/src setup-submodules && \
    make -C jbmc/src

RUN git clone https://github.com/plum-umd/java-sketch /opt/jsketch && cd /opt/jsketch/ && git checkout 

COPY . /opt/toshokan

ENV PATH=/usr/local/openjdk-8/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/rust/bin:/opt/sketch/sketch-frontend:/opt/cbmc/build/bin \
    SKETCH_HOME=/opt/sketch/sketch-frontend/runtime \
    SKETCH_FE=sketch \
    SKETCH_BE= \
    SKETCH_JAR= \
    JSKETCH_DIR=/opt/jsketch \
    JBMC_BIN=jbmc \ 
    CPROVER_JAR=/opt/cbmc/jbmc/lib/java-models-library/target/cprover-api.jar \
    JAVAC_BIN=javac

RUN cd /opt/toshokan && cargo build --examples

ENTRYPOINT ["/bin/bash"]