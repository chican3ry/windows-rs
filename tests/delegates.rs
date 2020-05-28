winrt::import!(
    dependencies
        os
    modules
        "windows.foundation"
        "windows.foundation.collections"
);

use winrt::ComInterface;

#[test]
fn non_generic() -> winrt::Result<()> {
    use windows::foundation::AsyncStatus;
    use windows::foundation::IAsyncAction;
    type Handler = windows::foundation::AsyncActionCompletedHandler;

    assert_eq!(
        Handler::iid(),
        winrt::Guid::from("A4ED5C81-76C9-40BD-8BE6-B1D90FB20AE7")
    );

    let d = Handler::default();
    assert!(d.is_null());

    let (tx, rx) = std::sync::mpsc::channel();

    let d = Handler::new(move |info, status| {
        tx.send(true).unwrap();
        assert!(info.is_null());
        assert!(status == AsyncStatus::Completed);
        Ok(())
    });

    // TODO: delegates are function objects (logically) ans we should be able
    // to call them without an explicit `invoke` method e.g. `d(args);`
    d.invoke(IAsyncAction::default(), AsyncStatus::Completed)?;

    assert!(rx.recv().unwrap());

    Ok(())
}

#[test]
fn generic() -> winrt::Result<()> {
    use windows::foundation::Uri;
    type Handler = windows::foundation::TypedEventHandler<Uri, i32>;

    // TODO: Handler::IID is not correct for generic types

    let d = Handler::default();
    assert!(d.is_null());

    let uri = Uri::create_uri("http://kennykerr.ca")?;
    let (tx, rx) = std::sync::mpsc::channel();

    let uri_clone = uri.clone();
    let d = Handler::new(move |sender, port| {
        tx.send(true).unwrap();
        assert!(uri_clone.as_raw() == sender.as_raw());

        // TODO: ideally primitives would be passed by value
        assert!(*port == 80);
        Ok(())
    });

    let port = uri.port()?;
    d.invoke(uri, port)?;

    assert!(rx.recv().unwrap());

    Ok(())
}

#[test]
fn event() -> winrt::Result<()> {
    use windows::foundation::collections::*;
    use windows::foundation::*;

    let set = PropertySet::new()?;
    let (tx, rx) = std::sync::mpsc::channel();

    let set_clone = set.clone();
    // TODO: Should be able to elide the delegate construction and simply say:
    // set.map_changed(|sender, args| {...})?;
    set.map_changed(
        MapChangedEventHandler::<winrt::HString, winrt::Object>::new(move |sender, args| {
            tx.send(true).unwrap();
            let set = set_clone.clone();
            let map: IObservableMap<winrt::HString, winrt::Object> = set.into();
            assert!(map.as_raw() == sender.as_raw());
            assert!(args.key()? == "A");
            assert!(args.collection_change()? == CollectionChange::ItemInserted);
            Ok(())
        }),
    )?;

    set.insert("A", PropertyValue::create_uint32(1)?)?;

    assert!(rx.recv().unwrap());

    Ok(())
}