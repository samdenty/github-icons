import { OptionProps } from 'react-select';
import AsyncSelect from 'react-select/async';
import { IconButton } from './IconButton';
import styled from '@emotion/styled';

interface Package {
  name: string;
}

const StyledOption = styled(IconButton)<{
  focused: boolean;
}>`
  background: ${(props) => (props.focused ? 'red' : 'none')};

  > img {
    height: 40px;
    width: 40px;
  }
`;

function Option({
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
      slug={`npm/${data.name}`}
      focused={isFocused}
      aria-disabled={isDisabled}
      {...(innerProps as any)}
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

async function loadOptions(query: string): Promise<Package[]> {
  const data = await fetch(
    `https://registry.npmjs.org/-/v1/search?text=${encodeURIComponent(query)}`
  ).then((res) => res.json());

  return data.objects.map((result: any) => result.package);
}

export function Search() {
  return (
    <AsyncSelect<Package>
      cacheOptions
      loadOptions={loadOptions}
      components={{ Option }}
    />
  );
}
