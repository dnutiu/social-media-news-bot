# Playbook

The ansible playbook can be used to deploy the social media news bot.
The deployment is done by either docker or podman containers and the compose plugin.

You can configure the project to deploy only the services that you want: scraper, mastodon bot or bluesky bot.

## Dependencies

You will need to install the following dependencies on your system to run the playbook:

```shell
	sudo dnf install ansible
	ansible-galaxy collection install community.general
	ansible-galaxy collection install containers.podman
	ansible-galaxy collection install community.docker
	ansible-galaxy collection install ansible.posix
```