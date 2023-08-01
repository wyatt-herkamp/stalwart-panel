# Stalwart Panel
An Unofficial Stalwart Panel. 
### Requirements
- Stalwart 0.3

### Goals
- Add, Remove, and Modify Accounts
- Reset Passwords
- Add and Modify Domains


### Notes

- This is a work in progress.
- This has a different database structure than Stalwart uses by default. This also does not support Sqlite. 
- All passwords are hashed with Argon2.
### To the Stalwart Development Team

If you are interested in adopting this project, please contact me. I would be happy to help you with it.


#### Caddyfile
```text
<your_domain> {
    # Handle Normal API Calls
    handle /api/* {
		reverse_proxy 127.0.0.1:5312
	}
	# Handle Frontend API Calls
    handle /frontend-api/* {
		reverse_proxy 127.0.0.1:5312
	}
	# Handle Frontend
	handle {
		root * /opt/stalwart-panel/frontend
		try_files {path} /index.html
		file_server
	}
}
```