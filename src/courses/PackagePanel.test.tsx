import { fireEvent,render,screen,waitFor } from '@testing-library/react';
import { expect,it,vi } from 'vitest';
import userEvent from '@testing-library/user-event';
import { PackagePanel } from './PackagePanel';
const review={reviewId:'opaque',kind:'nativeFull' as const,manifest:{kind:'nativeFull',scope:'selectedCourse',sourceCourseId:'a',complete:false,missingAssets:['digest'],omissions:['other courses'],tableCounts:{resource_revisions:2}}};
it('requires explicit incomplete confirmation and preserves exact retry identity/title',async()=>{
 const port={exportPackage:vi.fn(),inspect:vi.fn().mockResolvedValue(review),confirm:vi.fn().mockRejectedValueOnce({code:'outcome_unconfirmed'}).mockResolvedValueOnce({kind:'nativeFull',report:{sourceCourseId:'a',courseId:'new',complete:false,missingAssets:['digest'],omissions:['other courses']}}),discard:vi.fn()};
 const onImported=vi.fn();render(<PackagePanel port={port} onImported={onImported}/>);const user=userEvent.setup();
 await user.click(screen.getByRole('button',{name:'Choose package to inspect'}));
 expect(await screen.findByRole('region',{name:'Package review'})).toHaveTextContent('other courses');
 await user.type(screen.getByLabelText('New course title'),'Recovered');
 expect(screen.getByRole('button',{name:'Confirm import into new course'})).toBeDisabled();
 await user.click(screen.getByLabelText('Allow incomplete recovery'));
 await user.click(screen.getByRole('button',{name:'Confirm import into new course'}));
 await user.click(await screen.findByRole('button',{name:'Retry same package import'}));
 expect(port.confirm.mock.calls[1]).toEqual(port.confirm.mock.calls[0]);
 expect(await screen.findByText(/Created new course new/)).toBeVisible();expect(onImported).toHaveBeenCalled();
});
it('keeps originals opt-in, reports cancellation and never promotes content packages to full recovery',async()=>{
 const port={exportPackage:vi.fn().mockResolvedValue(null),inspect:vi.fn().mockResolvedValue({...review,kind:'portableContent',manifest:{...review.manifest,kind:'portableContent',fullRecovery:false}}),confirm:vi.fn(),discard:vi.fn().mockResolvedValue(undefined)};
 render(<PackagePanel port={port}/>);const user=userEvent.setup();
 await user.click(screen.getByRole('button',{name:'Export shareable content package'}));
 expect(port.exportPackage).toHaveBeenCalledWith('portableContent',false,false);
 expect(await screen.findByText('Package export cancelled.')).toBeVisible();
 await user.click(screen.getByRole('button',{name:'Choose package to inspect'}));
 expect(await screen.findByRole('region',{name:'Package review'})).toHaveTextContent('Content only: does not provide full recovery');
 await user.click(screen.getByRole('button',{name:'Discard package review'}));
 await waitFor(()=>expect(screen.queryByRole('region',{name:'Package review'})).not.toBeInTheDocument());
});
it('shows picker errors and supports a fresh inspection without losing the course-bound port',async()=>{
 const port={exportPackage:vi.fn(),inspect:vi.fn().mockRejectedValueOnce({code:'invalid_request'}).mockResolvedValueOnce(null),confirm:vi.fn(),discard:vi.fn()};
 render(<PackagePanel port={port}/>);const user=userEvent.setup();
 await user.click(screen.getByRole('button',{name:'Choose package to inspect'}));
 expect(await screen.findByRole('alert')).toBeVisible();
 await user.click(screen.getByRole('button',{name:'Choose package to inspect'}));
 expect(await screen.findByText('Package inspection cancelled.')).toBeVisible();
 expect(screen.queryByRole('alert')).not.toBeInTheDocument();
});

it('requires explicit opt-in to export private study history', async () => {
 const port={exportPackage:vi.fn().mockResolvedValue(null),inspect:vi.fn(),confirm:vi.fn(),discard:vi.fn()};
 render(<PackagePanel port={port}/>);
 const history=screen.getByRole('checkbox',{name:'Include my study responses and assessment history in shareable package'});
 expect(history).not.toBeChecked();
 fireEvent.click(history);
 expect(screen.getByText(/saved study drafts, earlier draft snapshots/)).toBeInTheDocument();
 fireEvent.click(screen.getByRole('button',{name:'Export shareable content package'}));
 await waitFor(()=>expect(port.exportPackage).toHaveBeenCalledWith('portableContent',false,true));
});
