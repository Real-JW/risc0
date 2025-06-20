eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_rsa_jw
ssh-add -l
ssh -T git@github.com
