import styled from '@emotion/styled';
import { IconButton } from '../IconButton/IconButton';
import { OptionProps } from 'react-select';
import { Package } from './Search';

interface StyledOptionProps {
  focused: 1 | 0;
}

const StyledOption = styled(IconButton)<StyledOptionProps>`
  --size: 30px;
  padding: 5px;
  background: ${(props) => (props.focused ? 'red' : 'none')};
`;

export function Option({
  data,
  isDisabled,
  isFocused,
  innerRef,
  innerProps,
}: OptionProps<Package>) {
  return (
    <StyledOption
      key={data.name}
      ref={innerRef as any}
      type="npm"
      slug={data.name}
      focused={isFocused ? 1 : 0}
      showBadge
      aria-disabled={isDisabled}
      {...(innerProps as {})}
      onMouseDown={(e) => {
        if (e.button === 1) {
          e.preventDefault();
          window.open(e.currentTarget.href);
        }
      }}
    >
      {data.name}
    </StyledOption>
  );
}
