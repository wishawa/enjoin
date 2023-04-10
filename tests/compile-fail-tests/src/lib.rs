/**
```compile_fail
async {
    struct NotCopy;
    let mut nc = NotCopy;
    enjoin::join_auto_borrow!(
        {
            nc
        },
        {
            {
                let _a = &mut nc;
            }
            3
        }
    );
};
```
```
async {
    #[derive(Clone, Copy)]
    struct IsCopy;
    let mut nc = IsCopy;
    enjoin::join_auto_borrow!(
        {
            nc
        },
        {
            {
                let _a = &mut nc;
            }
            3
        }
    );
};
```
*/
struct _ConflictWithOwned;

/**
```compile_fail
async {
    let mut x = 5;
    enjoin::join_auto_borrow!(
        {
            let z = &mut x;
            core::future::ready(3).await;
            drop(z);
        },
        {
            let z = &mut x;
            core::future::ready(3).await;
            drop(z);
        }
    );
};
```
```
async {
    let mut x = 5;
    enjoin::join_auto_borrow!(
        {
            let z = &mut x;
            core::future::ready(3).await;
            drop(z);
        },
    );
};
```
```
async {
    let mut x = 5;
    enjoin::join_auto_borrow!(
        {
            let z = &mut x;
            drop(z);
            core::future::ready(3).await;
        },
        {
            let z = &mut x;
            drop(z);
            core::future::ready(3).await;
        }
    );
};
```
*/
struct _BorrowAcrossYieldPoint;
