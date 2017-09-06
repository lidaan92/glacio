from fabric.api import sudo, cd, task, run, local, env

REMOTE_DIRECTORY = "/var/www/glacio"

env.hosts = ["lidar.io"]
env.use_ssh_config = True

@task
def deploy():
    push()
    update()
    restart()

@task
def push():
    local("git push")

@task
def update():
    with cd(REMOTE_DIRECTORY):
        run("git pull")
        run("cargo build --release --all")
        run("cargo test --all")

@task
def restart():
    sudo("supervisorctl restart glacio-api")
