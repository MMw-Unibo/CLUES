FROM quay.io/jupyter/minimal-notebook:2025-02-10

ARG NAME="Alessandro Aimi"
ARG EMAIL="alessandro.aimi@unibo.it"
LABEL Maintainer="${NAME} <${EMAIL}>"
LABEL Description="Docker image for experiments with ns-3"

USER root

# Install ns-3 dependencies
RUN apt-get update --yes && \
    apt-get install --yes --no-install-recommends \
    g++ \
    cmake \
    ninja-build \
    ccache \
    # Matplotlib deps
    dvipng \
    cm-super \
    # Utility
    htop && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

USER ${NB_UID}

# Install SciencePlots, Seaborn and SEM (from repo)
RUN mamba install -y \
    'SciencePlots' \
    'seaborn' && \
    mamba clean -afy && \
    git clone https://github.com/signetlabdei/sem.git && \
    git -C "/home/${NB_USER}/sem" checkout 6a39f0a1f1d0458fd5ee843a221876bd19fe4dc0 && \
    pip install --no-cache-dir -e "/home/${NB_USER}/sem" && \
    fix-permissions "${CONDA_DIR}" && \
    fix-permissions "/home/${NB_USER}"

# Install ns-3 and lorawan module
ENV NS3DIR="/home/${NB_USER}/ns-3-dev"
RUN git clone --depth 1 -b ns-3.41 https://gitlab.com/nsnam/ns-3-dev.git "${NS3DIR}" && \
    git -C "${NS3DIR}" clone --depth 1 -b v0.3.1 https://github.com/signetlabdei/lorawan.git src/lorawan && \
    git -C "${NS3DIR}" submodule add https://github.com/signetlabdei/lorawan.git src/lorawan && \
    "${NS3DIR}"/ns3 configure -d optimized --out=build/optimized --enable-examples --enable-modules lorawan && \
    "${NS3DIR}"/ns3 build && \
    fix-permissions "${NS3DIR}"

# Patch lorawan separately to reduce image rebuild time
COPY --chown=${NB_UID}:${NB_GID} lorawan.patch "/home/${NB_USER}/"
RUN git -C "${NS3DIR}/src/lorawan" -c user.name="${NAME}" -c user.email="${EMAIL}" am "/home/${NB_USER}/lorawan.patch" && \
    git -C "${NS3DIR}" -c user.name="${NAME}" -c user.email="${EMAIL}" commit -a -m "Add lorawan submodule with lora-matter patch" && \
    "${NS3DIR}"/ns3 build && \
    fix-permissions "${NS3DIR}"

COPY --chmod=755 run-all.sh /usr/local/bin/
