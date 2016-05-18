* Summary
   - Web test tool write in rust.
    Just for fun and exercise.

* Examples
    - this code will get bing 100 times
    ```
   ./burstweb http://www.bing.com -t 100
   ```
   
    - this code will post "a=1&b=2" 100 times
    
    ```
   ./burstweb http://example.com/api/add -p "a=1&b=2" -t 100
   ```
