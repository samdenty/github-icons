import { InputProps, components } from 'react-select';
import { Package } from './Search';

export function Input(props: InputProps<Package>) {
  return <components.Input {...props} isHidden={false} />;
}
