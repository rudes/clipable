@echo off
sc stop Clipable
sc delete Clipable
sc create Clipable start=auto binPath= %cd%"\target\debug\clipable.exe"
sc description Clipable "Uploads clips from Documents/Clipable to Streamable.com"
sc start Clipable
PAUSE