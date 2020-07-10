FROM ubuntu:18.04

MAINTAINER Bastian Bloessl <mail@bastibl.net>

ARG USER=gnuradio

ENV DEBIAN_FRONTEND noninteractive
RUN apt-get update
RUN apt-get install -y git vim wget unzip sudo cmake libtool build-essential pkg-config autogen python-dev python-mako python-six swig3.0 python3-mako python3-numpy

RUN apt-get install -y build-essential cmake gnuradio libuhd-dev libfftw3-3 swig curl

RUN groupadd -g 1000 -r $USER
RUN useradd -u 1000 -g 1000 --create-home -r $USER

RUN echo "$USER:$USER" | chpasswd

#Make sudo passwordless
RUN mkdir -p /etc/sudoers.d
RUN echo "${USER} ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/90-$USER
RUN usermod -aG sudo $USER
RUN usermod -aG plugdev $USER

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.44.1

RUN set -eux; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='ad1f8b5199b3b9e231472ed7aa08d2e5d1d539198a15c5b1e53c746aad81d27b' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='6c6c3789dabf12171c7f500e06d21d8004b5318a5083df8b0b02c0e5ef1d017b' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='26942c80234bac34b3c1352abbd9187d3e23b43dae3cf56a9f9c1ea8ee53076d' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='27ae12bc294a34e566579deba3e066245d09b8871dc021ef45fc715dced05297' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.21.1/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

RUN apt-get -y install libqt5widgets5 libqwt-qt5-dev qt5-default gr-iio

USER $USER

RUN echo "source $HOME/.cargo/env" >> /home/$USER/.bashrc

RUN mkdir -p /home/$USER/src
WORKDIR /home/$USER/src
RUN git clone  https://github.com/ucsdsysnet/sparsdr.git
WORKDIR /home/$USER/src/sparsdr/gr-sparsdr
RUN mkdir -p /home/$USER/src/sparsdr/gr-sparsdr/build
WORKDIR /home/$USER/src/sparsdr/gr-sparsdr/build
RUN cmake ..
RUN make
RUN sudo make install
RUN sudo ldconfig

WORKDIR /home/$USER/src/sparsdr/reconstruct
RUN cargo install --path .

ENTRYPOINT ["/bin/bash"]
