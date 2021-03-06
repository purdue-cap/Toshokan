FROM openjdk:8-jdk as jbmc_builder

RUN apt update -y && apt install -y \
    build-essential git maven bison flex

RUN git clone https://github.com/diffblue/cbmc /opt/cbmc && \
    cd /opt/cbmc && make -C src DOWNLOADER=wget minisat2-download -j8 && \
    make -C jbmc/src setup-submodules -j8 && \
    make -C jbmc/src -j8

FROM openjdk:8-jdk as sketch_builder

RUN apt update -y && apt install -y \
    build-essential wget bison flex 

RUN wget https://people.csail.mit.edu/asolar/sketch-1.7.6.tar.gz && \
    tar xvf sketch-1.7.6.tar.gz ; \
    rm sketch-1.7.6.tar.gz && \
    mv sketch-1.7.6 /opt/sketch && \
    cd /opt/sketch/sketch-backend && \
    ./configure && \
    make -j8

FROM openjdk:8-jdk as jsketch_builder

RUN apt update -y && apt install -y \
    git maven build-essential

RUN git clone https://github.com/plum-umd/java-sketch /opt/jsketch && \
    ln -sf /usr/local/openjdk-8/bin/javac /usr/bin/javac && \
    ln -sf /usr/local/openjdk-8/bin/java /usr/bin/java

RUN cd /opt/jsketch/jskparser && make p; make j && rm -rf /opt/jsketch/.git && rm -rf /opt/jsketch/test

FROM openjdk:8-jdk

RUN apt update -y && apt install -y \
    curl build-essential \
    python2 python3-matplotlib python-is-python2

COPY --from=sketch_builder /opt/sketch/sketch-backend/src/SketchSolver/cegis /opt/sketch/sketch-frontend/sketch /opt/sketch/sketch-frontend/sketch-1.7.6-noarch.jar /opt/sketch
COPY --from=sketch_builder /opt/sketch/sketch-frontend/runtime /opt/sketch/runtime

COPY --from=jbmc_builder /opt/cbmc/jbmc/src/jbmc/jbmc /usr/local/bin/
COPY --from=jbmc_builder /opt/cbmc/jbmc/lib/java-models-library/target/cprover-api.jar /usr/local/share/

ENV RUSTUP_HOME=/opt/rust \
    CARGO_HOME=/opt/rust \
    PATH=/usr/local/openjdk-8/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/rust/bin:/opt/sketch \
    SKETCH_HOME=/opt/sketch/runtime \
    SKETCH_FE=sketch \
    SKETCH_BE=cegis \
    SKETCH_JAR=/opt/sketch/sketch-1.7.6-noarch.jar \
    JSKETCH_DIR=/opt/jsketch \
    JBMC_BIN=jbmc \ 
    CPROVER_JAR=/usr/local/share/cprover-api.jar \
    JAVAC_BIN=javac

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --no-modify-path

# COPY --from=jsketch_builder /opt/jsketch /opt/jsketch
RUN apt update -y && apt install -y \
    git maven build-essential

RUN git clone https://github.com/plum-umd/java-sketch /opt/jsketch && \
    ln -sf /usr/local/openjdk-8/bin/javac /usr/bin/javac && \
    ln -sf /usr/local/openjdk-8/bin/java /usr/bin/java

RUN cd /opt/jsketch/jskparser && make p; make j

COPY . /opt/toshokan

RUN ln -sf /usr/local/openjdk-8/bin/javac /usr/bin/javac && \
    ln -sf /usr/local/openjdk-8/bin/java /usr/bin/java

ENTRYPOINT ["/bin/bash"]