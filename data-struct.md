# redis:  
service_server_urls{  
  push_server_urls:id{  
    (id,service_group,live_type,url)//集合  
    ...  
  }  
  ws_server_urls:id{  
    (id,service_group,live_type,url)//集合  
    ...  
  }  
  ...  
}  

# ClientPushMsg
push to ws server:String "<MsgType>,<ClientUuid>,<Msg>,<KafkaKey(会话id)>"
result:String "<Result>"
# WsServerPushMsg
push to push server:String "<Time>,<ClientUuid>,<Msg>,<KafkaTopic>,<KafkaKey>"
response to client:String "<Result>"

# Grpc PushServerMsg 
push to kafka:String "<Time>,<ClientUuid>,<Msg>"
response to ws server:String "<ResultType>,<Result>"

# Kafka 
push to consumer:String "<Time>,<ClientUuid>,<Msg>"

# Consumer
push to db:String "<Time>,<ClientUuid>,<Msg>"