use iota_lib_rs::prelude::iota_client;

use iota_streams::app::transport::Transport;
use iota_streams::app::transport::tangle::client::SendTrytesOptions;
use iota_streams::app_channels::api::tangle::{Author, Subscriber, Address,};
use iota_streams::app_channels::message;
use iota_streams::protobuf3::types::Trytes;
use iota_streams::core::tbits::Tbits;

use failure::{ensure,Fallible};
use std::{thread, time::Duration, str::FromStr};


fn main() -> Fallible<()> {


    let mut client = iota_client::Client::new("https://nodes.devnet.iota.org:443");
    let mut send_opt = SendTrytesOptions::default();
    send_opt.min_weight_magnitude = 9;
    send_opt.local_pow = false;



    let mut author = Author::new("EWQPWQPQMXKRBMKTRZTZ", 3, true);

    let announcement_message = author.announce().unwrap();
    let announcement_tag = announcement_message.link.msgid.to_string();
    println!("A: Announcement message generated with tag: {}",announcement_tag);

    client.send_message_with_options(&announcement_message, send_opt).unwrap();
    println!("A: Announcement message sent");



    thread::sleep(Duration::from_secs(20));


    
    let mut subscriber = Subscriber::new("FRE9DWWEEW9", true);

    let channel_address = author.channel_address().to_string();
    
    let announcement_link = Address::from_str(&channel_address, &announcement_tag).unwrap();


    let response_list = client.recv_messages_with_options(&announcement_link, ()).unwrap();
    let response = &response_list[0];

    println!("S: Announcement message found at tag: {}", &response.link.msgid);
    let header = response.parse_header()?;
    ensure!(header.check_content_type(message::announce::TYPE));
    subscriber.unwrap_announcement(header.clone()).unwrap();
    println!("S: Announcement message unwraped, we can now read public messages");

    

    let subscription_message = subscriber.subscribe(&announcement_link).unwrap();
    let subscribe_tag = subscription_message.link.msgid.to_string();
    println!("S: Subscription message generated with tag {}", &subscribe_tag);

    client.send_message_with_options(&subscription_message, send_opt).unwrap();
    println!("S: Subscription message sent");



    thread::sleep(Duration::from_secs(20));



    let subscription_link = Address::from_str(&channel_address, &subscribe_tag).unwrap();

    let response_list = client.recv_messages_with_options(&subscription_link, ()).unwrap();
    let response = &response_list[0];
    println!("A: Subscription message found at tag: {}", &response.link.msgid);

    let header = response.parse_header()?;
    ensure!(header.check_content_type(message::subscribe::TYPE));
    author.unwrap_subscribe(header.clone()).unwrap();
    println!("A: Subscription message unwraped, we can now share keyload");


    let keyload_message = author.share_keyload_for_everyone(&announcement_link).unwrap();
    let keyload_tag = keyload_message.link.msgid.to_string();
    println!("A: Keyload message generated with tag: {}", &keyload_tag);
    client.send_message_with_options(&keyload_message, send_opt).unwrap();
    println!("A: Keyload message sent");



    thread::sleep(Duration::from_secs(20));



    let keyload_link = Address::from_str(&channel_address, &keyload_tag).unwrap(); 

    let response_list = client.recv_messages_with_options(&keyload_link, () ).unwrap();
    let response = &response_list[0];
    println!("S: Keyload message found at tag: {}", &response.link.msgid);

    let header = response.parse_header()?;
    ensure!(header.check_content_type(message::keyload::TYPE));
    subscriber.unwrap_keyload(header.clone()).unwrap();
    println!("S: Keyload message unwraped, we can now read maskes messages");
        

    let public_payload = Trytes(Tbits::from_str("I9AM9PUBLIC").unwrap());
    let empty_masked_payload = Trytes(Tbits::from_str("").unwrap());

    let signed_public_message = author.sign_packet(&announcement_link, &public_payload, &empty_masked_payload).unwrap();
    let signed_public_tag = signed_public_message.link.msgid.to_string();
    println!("A: Signed public message generated with tag: {}",signed_public_tag);

    client.send_message_with_options(&signed_public_message, send_opt)?;
    println!("A: Signed public message sent");


    let empty_public_payload = Trytes(Tbits::from_str("").unwrap());
    let masked_payload = Trytes(Tbits::from_str("I9AM9HIDDEN").unwrap());

    let signed_masked_message = author.sign_packet(&keyload_link, &empty_public_payload, &masked_payload)?;
    let signed_masked_tag = signed_masked_message.link.msgid.to_string();
    println!("A: Signed masked message generated with tag: {}",signed_masked_tag);

    client.send_message_with_options(&signed_masked_message, send_opt).unwrap();
    println!("A: Signed masked message sent");

    
    
    thread::sleep(Duration::from_secs(20));



    let signed_public_link = Address::from_str(&channel_address, &signed_public_tag).unwrap();

    let response_list = client.recv_messages_with_options(&signed_public_link, ()).unwrap();
    let response = &response_list[0];
    println!("S: Signed public message found at tag: {}", &response.link.msgid);

    let header = response.parse_header()?;
    ensure!(header.check_content_type(message::signed_packet::TYPE));
    let (public_payload, _masked_payload) = subscriber
        .unwrap_signed_packet(header.clone())
        .unwrap();
    println!("S: Unwraped Signed public message, found public: {}", public_payload);


    let signed_masked_link = Address::from_str(&channel_address, &signed_masked_tag).unwrap();

    let response_list = client.recv_messages_with_options(&signed_masked_link, ()).unwrap();
    let response = &response_list[0];
    println!("S: Signed masked message found at tag: {}", &response.link.msgid);

    let header = response.parse_header()?;
    ensure!(header.check_content_type(message::signed_packet::TYPE));
    let (_public_payload, masked_payload) = subscriber   
        .unwrap_signed_packet(header.clone())
        .unwrap();
    println!("S: Unwraped Signed masked message, found masked: {}", masked_payload);

    Ok(())
}
