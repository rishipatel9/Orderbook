Frontend ----HTTP Order----> Actix API ----> Redis Queue (orders)
   ^                               |                 
   |                               v
   | <--- WS Updates ---- Redis Pub/Sub <---- Worker (matching engine)
   |                                                     |
   |                                                     v
   |                                              DB (orders, trades, snapshots)
