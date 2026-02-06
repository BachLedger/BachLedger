sudo cp yzchain.service  /etc/systemd/system
sudo systemctl daemon-reload
sudo systemctl start yzchain
sudo systemctl enable yzchain
sudo systemctl status yzchain
