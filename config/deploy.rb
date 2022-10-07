# frozen_string_literal: true

# config valid for current version and patch releases of Capistrano
lock "~> 3.17.1"

set :application, "logsnarf"
set :repo_url, "git@git.sr.ht:~paul/logsnarf-rb-new"
# Default branch is :master
# ask :branch, `git rev-parse --abbrev-ref HEAD`.chomp
set :branch, "main"

# Default deploy_to directory is /var/www/my_app_name
set :deploy_to, "/var/www/logsnarf"

# Default value for :format is :airbrussh.
# set :format, :airbrussh

# You can configure the Airbrussh format using :format_options.
# These are the defaults.
# set :format_options, command_output: true, log_file: "log/capistrano.log", color: :auto, truncate: :auto

# Default value for :pty is false
# set :pty, true

# Default value for :linked_files is []
# append :linked_files, "config/database.yml"

# Default value for linked_dirs is []
# append :linked_dirs, "log", "tmp/pids", "tmp/cache", "tmp/sockets", "public/system"

# Default value for default_env is {}
# set :default_env, { path: "/opt/ruby/bin:$PATH" }

# Default value for local_user is ENV['USER']
# set :local_user, -> { `git config user.name`.chomp }
set :local_user, "logsnarf"

# Default value for keep_releases is 5
# set :keep_releases, 5

# Uncomment the following to require manually verifying the host key before first deploy.
# set :ssh_options, verify_host_key: :secure
set :ssh_options, forward_agent: true

# set :sentry_host, 'https://my-sentry.mycorp.com' # https://sentry.io by default
# set :sentry_api_token, '0123456789abcdef0123456789abcdef'
set :sentry_organization, "scalar"     # fetch(:application) by default
set :sentry_project,      "logsnarf-2" # fetch(:application) by default
set :sentry_repo_integration, false
# set :sentry_repo, 'my-org/my-proj' # computed from repo_url by default

before "deploy:starting", "sentry:validate_config"
after "deploy:published", "sentry:notice_deployment"

append :linked_dirs, ".bundle"

namespace :deploy do
  task :fix_permisssions do
    on roles(:app) do
      # execute "chgrp -R logsnarf /var/www/logsnarf"
      # execute "chgrp -R logsnarf #{release_path}"
    end
  end

  desc "Restart application"
  task :restart do
    on roles(:app) do
      execute "install -o logsnarf -g logsnarf -m 644 #{release_path}/ops/templates/logsnarf.service /var/www/logsnarf/.config/systemd/user/logsnarf.service"
      execute "install -o root -g root -m 644 #{release_path}/ops/templates/restart-logsnarf.service /var/www/logsnarf/.config/systemd/user/restart-logsnarf.service"
      execute "install -o root -g root -m 644 #{release_path}/ops/templates/restart-logsnarf.timer /var/www/logsnarf/.config/systemd/user/restart-logsnarf.timer"
      execute "sudo -u logsnarf systemctl --user daemon-reload"
      execute "sudo -u logsnarf systemctl --user enable restart-logsnarf.timer"
      execute "sudo -u logsnarf systemctl --user restart restart-logsnarf.timer"
      execute "sudo -u logsnarf systemctl --user enable logsnarf.service"
      execute "sudo -u logsnarf systemctl --user restart logsnarf.service"
      execute "sudo -u logsnarf systemctl --user status logsnarf.service"
    end
  end
end

before "deploy:restart", "deploy:fix_permisssions"
after "deploy:publishing", "deploy:restart"
