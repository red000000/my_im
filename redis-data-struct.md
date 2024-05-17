# redis:  
service_server_urls{  
  push_server_urls:id{  
    (id,service_group,live_type,url)//集合  
    ...  
  }  
  ...  
}  

# ClientPushMsg
String "<Msg>"

# WsServerPushMsg
String "<Time>,<Msg>"

# (grpc)PushServerMsg
String "<Time>,<Msg>,<KafkaTopic>,<KafkaKey>"

