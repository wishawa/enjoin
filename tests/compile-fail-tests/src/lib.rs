/**
```compile_fail
async {
    struct NotCopy;
    let mut nc = NotCopy;
    enjoin::join!(
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
    enjoin::join!(
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
