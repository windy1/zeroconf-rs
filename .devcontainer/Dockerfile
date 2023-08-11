FROM ubuntu:jammy

RUN apt-get update && apt-get install -y curl git

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y

RUN apt-get install -y xorg-dev libxcb-shape0-dev libxcb-xfixes0-dev clang avahi-daemon libavahi-client-dev
