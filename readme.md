# nativefier 

> Create native apps for your favourite websites. 

## Key Components 

- [ ] Mechanism for specifying behaviour type: generator or generated.  
    - [ ] MacOS
        1. [x] Resolve absolute path to binary  
        1. [x] If binary is inside `.app` directory then we are in generated mode else, we are in generator mode (default)    
        1. [ ] Use wrapper shell script to call the binary and pass in url as a flag  
    - [x] Windows 
        1. [x] Name of executale file.

- [ ] Detect appropriate icon for website. 
    1. Infer icon from site content  
    1. Download  
    1. Add to final binary  
