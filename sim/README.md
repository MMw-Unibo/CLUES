# Simulation experiments suite

Follow the instructions below to reproduce our simulation experiments and plots using Docker Compose. Key files:

- The file [`build/lorawan.patch`](build/lorawan.patch) contains our extension of [ns-3](https://www.nsnam.org/)'s [lorawan module](https://apps.nsnam.org/app/lorawan);

- Simulation campaigns can be found under the [`experiments/`](experiments/) directory as IPython notebooks following the convention `experiments/<name>/<name>.ipynb`.

## Prerequisites

In order to run the experiments suite you will need GNU/Linux with Docker Engine and Docker Compose.

Tested on a Ubuntu 22.04.5 LTS virtual machine with Docker Engine v27.3.1 and Docker Compose v2.29.7.

Before running any `docker compose` command, please run

```bash
echo "DOCKER_USER=$(id -u):$(id -g)" > .env
```

from this directory. This allows the container to set the right permissions on output files under the [`experiments/`](experiments/) directory.

***All the commands that follow are to be run from the same directory of this README unless specified otherwise.***

## Running

> Beware that simulations can take many hours. We run the experiments on a VM with 64 vCores and 126 GB of RAM, on a host machine with 2 Intel(R) Xeon(R) Gold 6238R CPUs @ 2.20GHz (28 cores, 56 threads each), and it took 4 hours and 10 minutes. Estimate your time based on the number of cores at your disposal.

Docker will automatically download a pre-built docker image from [docker hub](https://hub.docker.com) when launching one of the following commands. If you prefer, you can build it yourself with `docker compose build` but it can take time.

Available options:

1. **Run the experiments step by step with IPython notebooks**: The following command starts a Jupyter Lab server at `localhost:8888`. The command also prints the local server URL with the required access token; paste it in your browser to access the Jupyter Lab GUI. Notebooks are located under `work/` in the Jupyter Lab GUI. For more info see [Jupyter](https://jupyter.org/).

    ```bash
    docker compose up -d && docker compose exec clues jupyter server list
    ```

    If you prefer using VSCode to run the notebooks, still run the command and copy the server URL. Then, add the Jupyter extension to VSCode and open one of the notebooks under [`experiments/`](experiments/). Use the ciped URL to attach to the existing Jupyter Server when prompted to. **Tip**: To simplify VSCode connection, you can setup a simple Jupyter Server password with `jupyter notebook password` in the container.

2. Run all simulation experiments notebooks in sequence non-interactively. This command creates a temporary container that is deleted once the process stops. The process can be interrupted and resumed by running the command again.

    ```bash
    docker compose run --rm clues run-all.sh
    ```

## Outputs overview

Output files are placed in the respective subfolders under [`experiments/`](experiments/). In order of creation, each experiment will have:

- `results/`: Each simulation produces multiple files stored here in a database managed by [SEM](https://github.com/signetlabdei/sem)
- `*.csv`: Aggregated multi-simulation results dataseta. Useful when working on plots to avoid having to parse `results/` multiple times.
- `plots/`: Final plots images.

Running the experiments with `run-all.sh` (second running option above) will also produce an additional `*.py` script version of each notebook.

## More information

All the build files for the experiment suite container are provided in the [`build/`](build/) directory.

### Working on simulation source code in the container environment

The container image we provide exposes an environment with most of the dependencies you need already installed. After running the container with the first command above, inside the Jupyter GUI (`localhost:8888`) you can directly edit the simulation source code found in `ns-3-dev/src/lorawan`. If you prefer a different IDE, you can also use VSCode to attach to the running container.

### Working on simulation source code locally

Otherwise, to work on the simulation source code locally you can reproduce the steps from line of 39 of the [Dockerfile](build/Dockerfile).

Here is a *draft* script to install and build the simulation code from this directory. Just make sure to have the [ns-3 build dependencies](https://www.nsnam.org/docs/release/3.41/installation/html/linux.html#requirements) installed on your system.

```bash
git clone --depth 1 -b ns-3.41 https://gitlab.com/nsnam/ns-3-dev.git && cd ns-3-dev && \
git clone --depth 1 -b v0.3.1 https://github.com/signetlabdei/lorawan.git src/lorawan && \
git -C "src/lorawan" am ../../../build/lorawan.patch && \
ns3 configure -d optimized --out=build/optimized --enable-examples --enable-modules lorawan && \
ns3 build && cd -
```

You can now run single simulations under the `ns-3-dev` directory; try `ns3 run "clues --help"`, or see the [ns-3 manual](https://www.nsnam.org/docs/release/3.41/manual/html/index.html) for more info.

#### Generating a patch and rebuilding the experiment suite

If you modify local files inside `ns-3-dev/src/lorawan` and you want to use the simulation suite docker image to run experiments, commit your changes in `ns-3-dev/src/lorawan` and, from there, run `git format-patch v0.3.1 --stdout > ../../../build/lorawan.patch` to update the patch. Now you should be able to run `docker compose build` in this directory to rebuild the image with your changes.
